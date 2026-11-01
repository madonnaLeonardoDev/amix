use reqwest;
use std::os::unix::io::AsRawFd;
use crate::lyrics::{LyricLine, request_lyric, get_current_lyrics};
use crate::{Commands};
use crate::apple_music_navigation::{Track};
use crate::paths::DAEMON_SOCKET_PATH;
use crossterm::{cursor, ExecutableCommand, terminal};
use std::io::{stdout, Write};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{Duration, interval};

async fn request_track_meta(stream: &mut UnixStream) -> Result<Option<Track>, String> {
        let request = wincode::serialize(&Commands::GetPlayingTrackMeta)
            .map_err(|e| format!("Failed serializing internal Command: {}", e))?;
        let len_prefix = (request.len() as u32).to_le_bytes();

        if let Err(e) = stream.write_all(&len_prefix).await {
            return Err(format!("Could not write to stream length prefix: {}", e))
        }
        if let Err(e) = stream.write_all(&request).await {

            return Err(format!("Could not write to stream payload: {}", e))
        }

        let mut len_buf = [0u8; 4];
        if let Err(e) = stream.read_exact(&mut len_buf).await {
            return Err(format!("Error reading len buf: {}", e))
        }
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        if let Err(e) = stream.read_exact(&mut buf).await {
            return Err(format!("Error reading payload buf: {}", e))
        }

        let current_track: Option<Track> = wincode::deserialize(&buf)
            .map_err(|e| e.to_string())?;

        Ok(current_track)
}




pub async fn run_player_loop(lyrics_enabled: bool) -> Result<(), String> {
let mut ticker = interval(Duration::from_millis(100));
    let mut rendered_track_id: Option<String> = None;

    let mut lyrics: Vec<LyricLine> = Vec::new();

    loop {
        let mut stream = match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Daemon connection failed: {}. Is it running?", e);
                ticker.tick().await;
                continue;
            }
        };

        let current_track = match request_track_meta(&mut stream).await {
            Ok(t) => t,
            Err(_) => continue
        };

        if let Some(track) = current_track {
            if rendered_track_id.is_none() {
                //SONG CHANGED
                if lyrics_enabled {
                    match request_lyric(&track).await {
                        Ok(v) => {
                            if let Some(l) = v {
                                lyrics = l;
                            } else {
                                lyrics.clear();
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }

                    };

                    let current_lyrics = get_current_lyrics(&lyrics, track.current_ms);

                    render_lyrics(&current_lyrics).await
                        .map_err(|e| e.to_string())?;
                }
                

                render_heavy_redraw(&track).await.map_err(|e| e.to_string())?;
            } else if let Some(id) = &track.id {
                //SAME SONG TIME ADVANCING
                if lyrics_enabled {

                    let current_lyrics = get_current_lyrics(&lyrics, track.current_ms);

                    render_lyrics(&current_lyrics).await
                        .map_err(|e| e.to_string())?;
                }

                if Some(id.clone()) == rendered_track_id { 
                    render_light_tick(&track).await.map_err(|e| e.to_string())?;
                } else {
                    //SONG CHANGED
                    if lyrics_enabled {
                    match request_lyric(&track).await {
                        Ok(v) => {
                            if let Some(l) = v {
                                lyrics = l;
                            } else {
                                lyrics.clear();
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }

                    };

                    let current_lyrics = get_current_lyrics(&lyrics, track.current_ms);

                    render_lyrics(&current_lyrics).await
                        .map_err(|e| e.to_string())?;
                }

                    render_heavy_redraw(&track).await.map_err(|e| e.to_string())?;
                }
            }
            rendered_track_id = track.id;
        }

        ticker.tick().await;
    }
}



async fn fetch_artwork(template_url: &str, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let url = template_url
        .replace("{w}", &width.to_string())
        .replace("{h}",&height.to_string());

    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    Ok(bytes.to_vec())
}

async fn render_heavy_redraw(track: &Track) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        libc::tcflush(std::io::stdin().as_raw_fd(), libc::TCIFLUSH);
    }
    
    let mut stdout = stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.flush()?;

    if let Some(v) = track.artwork.as_ref() {
    let art_bytes = fetch_artwork(&v.url, v.width, v.height).await?;
    let img = image::load_from_memory(&art_bytes)?;

    let conf = viuer::Config {
        width: Some(40),
        height: Some(20),
        x: 2, 
        y: 2,
        transparent: true,
        ..Default::default()
    };

    viuer::print(&img, &conf)?;
    }
    

    let text_x = 45;

    stdout.execute(cursor::MoveTo(text_x, 4))?;
    print!("TITLE:  {}", track.title.as_deref().unwrap_or("---"));

    stdout.execute(cursor::MoveTo(text_x, 6))?;
    print!("ARTIST: {}", track.artist.as_deref().unwrap_or("---"));

    stdout.execute(cursor::MoveTo(text_x, 8))?;
    print!("ALBUM:  {}", track.album.as_deref().unwrap_or("---"));
    render_light_tick(track).await?;
    stdout.flush()?;
    Ok(())
}

async fn render_light_tick(track: &Track) -> Result<(), Box<dyn std::error::Error>> {
    if let (None, None) = (track.current_ms, track.duration_ms){
        return Ok(())
    }

    let duration_ms: u32 = track.duration_ms.unwrap();
    let current_ms: u32 = track.current_ms.unwrap();

    let mut stdout = stdout();


    let dur = if duration_ms == 0 { 1 } else { duration_ms };
    let progress = (current_ms as f64 / dur as f64).clamp(0.0, 1.0);

    let bar_width = 30;
    let filled_count = (progress * bar_width as f64).round() as usize;
    let empty_count = bar_width - filled_count;

    let filled = "█".repeat(filled_count);
    let empty = "-".repeat(empty_count);


    let cur_sec = current_ms / 1000;
    let dur_sec = duration_ms / 1000;
    
    let cur_fmt = format!("{:02}:{:02}", cur_sec / 60, cur_sec % 60);
    let dur_fmt = format!("{:02}:{:02}", dur_sec / 60, dur_sec % 60);


    let text_x = 45;
    let text_y = 11;

    stdout.execute(cursor::MoveTo(text_x, text_y))?;
    
    print!("[{}{}]  {} / {}", filled, empty, cur_fmt, dur_fmt);

    stdout.flush()?;
    Ok(())
}


async fn render_lyrics(lyrics: &[&str; 3]) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let text_x = 45;
    let start_y = 14; // Positioned below the progress bar

    let [prev, current, next] = lyrics;

    // 1. Render Previous Line (Dimmed)
    stdout.execute(cursor::MoveTo(text_x, start_y))?;
    print!("\x1b[K\x1b[2m{}\x1b[0m", prev);

    // 2. Render Current Line (Bold/Bright with a pointer)
    stdout.execute(cursor::MoveTo(text_x, start_y + 2))?;
    print!("\x1b[K\x1b[1m> {}\x1b[0m", current);

    // 3. Render Next Line (Dimmed)
    stdout.execute(cursor::MoveTo(text_x, start_y + 4))?;
    print!("\x1b[K\x1b[2m{}\x1b[0m", next);

    stdout.flush()?;
    Ok(())
}