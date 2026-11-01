use reqwest;
use std::os::unix::io::AsRawFd;
use crate::lyrics::{LyricLine, get_current_lyrics, request_lyric};
use crate::{Commands};
use crate::browser_interface::{Track};
use crate::paths::DAEMON_SOCKET_PATH;
use crossterm::{cursor, ExecutableCommand, terminal};
use std::io::{Write, stdout};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{Duration, interval};

struct Display{
    track: Track,
    artwork_b: Option<Vec<u8>>,
    lyrics: Option<Vec<LyricLine>>,
}
impl Display {
    async fn update_track(&mut self) -> Result<(), String> {
        let mut stream = match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()).await {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Daemon connection failed: {}. Is it running?", e));
            }
        };

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

        let track: Option<Track> = wincode::deserialize(&buf)
            .map_err(|e| e.to_string())?;

        if let Some(t) = track {
            self.track = t;
        }
        Ok(())
    }

    async fn load_current() -> Result<Option<Self>, String> {
        let mut stream = match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()).await {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Daemon connection failed: {}. Is it running?", e));
            }
        };

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

        let track: Option<Track> = wincode::deserialize(&buf)
            .map_err(|e| e.to_string())?;

        if let Some(track) = track  {
        let mut artwork_b = None;

        if let Some(a) = track.artwork.as_ref() {
            match fetch_artwork(&a.url, a.width, a.height).await {
                Ok(a) => artwork_b = Some(a),
                Err(e) => return Err(e)
            }
        };

        let lyrics = request_lyric(&track).await?;
        

        return Ok(Some(
            Self { track, artwork_b, lyrics}
            ))
        }
        return Ok(None);
    }

    async fn load_next() -> Result<Option<Self>, String> {
        let mut stream = match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()).await {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Daemon connection failed: {}. Is it running?", e));
            }
        };

        let request = wincode::serialize(&Commands::GetNextTrackMeta)
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

        let track: Option<Track> = wincode::deserialize(&buf)
            .map_err(|e| e.to_string())?;

        if track.is_none() {
            return Ok(None);
        }
        let track = track.unwrap();

        let mut artwork_b = None;

        if let Some(a) = track.artwork.as_ref() {
            match fetch_artwork(&a.url, a.width, a.height).await {
                Ok(a) => artwork_b = Some(a),
                Err(e) => return Err(e)
            }
        };

        let lyrics = request_lyric(&track).await?;
        

        Ok(Some(
            Self {track, artwork_b, lyrics }
        ))
    }

    fn draw(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { libc::tcflush(std::io::stdin().as_raw_fd(), libc::TCIFLUSH) };
        
        let mut stdout = stdout();

        

            stdout.execute(terminal::Clear(terminal::ClearType::All))?;
            
            if let Some(v) = self.artwork_b.as_ref() {
                let img = image::load_from_memory(v)?;

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

        let mut draw_text_line = |y: u16, label: &str, value: Option<&str>| -> Result<(), Box<dyn std::error::Error>> {
            stdout.execute(cursor::MoveTo(text_x, y))?;
            print!("{:<50}", ""); 
            stdout.execute(cursor::MoveTo(text_x, y))?;
            print!("{}:  {}", label, value.unwrap_or("---"));
            Ok(())
        };

        draw_text_line(4, "TITLE", self.track.title.as_deref())?;
        draw_text_line(6, "ARTIST", self.track.artist.as_deref())?;
        draw_text_line(8, "ALBUM", self.track.album.as_deref())?;

        self.render_light_tick()?;
        stdout.flush()?;

        Ok(())
    }

    fn render_light_tick(&self) -> Result<(), Box<dyn std::error::Error>> {

        let (Some(_), Some(_)) = (self.track.duration_ms, self.track.current_ms) else {
            return Ok(());
        };

        let duration_ms: u32 = self.track.duration_ms.unwrap();
        let current_ms: u32 = self.track.current_ms.unwrap();

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

    fn render_lyrics(&self) -> Result<(), Box<dyn std::error::Error>> {

    let lyrics = match self.lyrics.as_ref() {
        Some(l) => get_current_lyrics(&l, self.track.current_ms),
        None => return Ok(())
    };
    
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
}

pub async fn run_player_loop(lyrics_enabled: bool) -> Result<(), String> {
let mut ticker = interval(Duration::from_millis(100));
    let mut prev_id: Option<String> = None;
    let mut display = Display::load_current().await?;
    loop {
            if let Some(d) = display.as_mut() {

                if prev_id!= d.track.id {
                //Heavy Redraw

                match Display::load_current().await?{
                    Some(new_d) => *d=new_d,
                    None => continue
                }

                if let Err(_) = d.draw() {
                    continue;
                };

                if lyrics_enabled {
                    if let Err(_) = d.render_lyrics() {
                        continue;
                    }
                }
                if let Err(_) = d.update_track().await {
                    continue;
                }

                prev_id = d.track.id.clone();


            } else {
                if let Err(_) = d.render_light_tick() {
                    continue;
                }

                if lyrics_enabled {
                    if let Err(_) = d.render_lyrics() {
                        continue;
                    }
                }

                if let Err(_) = d.update_track().await {
                    continue;
                }
            }
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