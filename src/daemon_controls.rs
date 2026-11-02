use thirtyfour::{WebDriver};
use crate::{Commands, browser_interface::*};

pub fn format_time(time_ms: u32) -> String {
    let tot_secs = time_ms / 1000;
    let seconds = tot_secs % 60;
    let minutes = tot_secs / 60;
    let hours = tot_secs / 3600;

    if hours > 0 {
        
        return format!("{:02}:{:02}",minutes, seconds);
        
    } else {
        return format!("{}:{:02}:{:02}", hours, minutes, seconds);
    }
}

impl Commands {
    pub async fn execute_daemon(self, driver:&WebDriver) -> Result<(Vec<u8>, bool /* requires persistent connection */), String> {
        match self {
        Commands::Search { stype, query, next, insta_play, library } => {
            let search_type = match stype.to_lowercase().as_str() {
                "song" | "s" => ItemType::Song,
                "album" | "a" => ItemType::Album,
                "playlist" | "p" => ItemType::Playlist,
                _ => {
                   return Err("Invalid search type (stype) argument".to_string());
                }
            };
            let joined_query = query.replace("-", " ");
            let search_res = driver.search(&joined_query, &search_type,library).await?;

            let result_tuple:(String, String) = match search_res {
                SearchResult::Track(track) => {
                    let id = if track.id.is_none() {
                    return Ok(("No Matches found".as_bytes().to_vec(), false));
                    } else {
                    track.id.clone().unwrap()
                    };

                    (track.pretty_display(), id)
            },
            SearchResult::Playlist(playlist) => {
                let id = if playlist.id.is_none() {
                    return Ok(("No Matches found".as_bytes().to_vec(), false));
                } else {
                    playlist.id.clone().unwrap()
                };

                (playlist.pretty_display(), id)
            },
            SearchResult::Album(album) => {
                let id = if album.id.is_none() {
                    return Ok(("No Matches Found".as_bytes().to_vec(), false));
                } else {
                    album.id.clone().unwrap()
                };

                (album.pretty_display(), id)

            }
            };
            if next{
                driver.play_next(&search_type, &result_tuple.1).await?;
            }
            if insta_play {
                driver.play_next(&search_type, &result_tuple.1).await?;
                driver.next_track().await?
            }
            if next && insta_play {
                return Err("Either Insta-Play (-p) or Play-Next (-n)".to_string());
            }
            Ok((result_tuple.0.as_bytes().to_vec(), false))
        },
        Commands::Current => {
            if let Some(track) = driver.get_current_track().await? {
                let title:String = if track.title.is_none(){
                    "Unkown Title".to_string()
                } else {
                    track.title.unwrap()
                };
                let artist: String = if track.artist.is_none() {
                    "Unkown Artist".to_string()
                } else {
                    track.artist.unwrap()
                };


                let album: String = if track.album.is_none(){
                    "Unkown Album".to_string()
                } else {
                    track.album.unwrap()
                };

                let duration: String = if track.duration_ms.is_none() {
                    "Unkown Duration".to_string()
                } else {
                    format_time(track.duration_ms.unwrap())
                };
                return Ok((format!(
                    "Track Playing:\n\r{}\n\r{} | {} ({})",
                    title, artist, album, duration 
                ).as_bytes().to_vec(), false))
            }

            Ok(("No Track playing".to_string().as_bytes().to_vec(), false))
        },
        Commands::LsPlaylists => {
            match driver.list_library_playlists().await {
            Ok(pl) => {
                let result: String = pl
                .iter()
                .map(|pl| pl.pretty_display())
                .collect();
                Ok((result.as_bytes().to_vec(), false))
            },
            Err(e) => return Err(e)
            }
        },
        Commands::Playid { stype, id, next } => {
            let item_type = match stype.to_lowercase().as_str() {
                "song" | "s" => ItemType::Song,
                "album" | "a" => ItemType::Album,
                "playlist" | "p" => ItemType::Playlist,
                _ => {
                    return Err("Invalid search type (stype) argument".to_string());
                }
            };

            let id = driver.play_next(&item_type, &id).await
                .map_err(|e| e.to_string())?;

                if !next {
                    driver.next_track().await?;
                    return Ok((format!("Playing Now: {}",id).as_bytes().to_vec(), false));
                }
            Ok((format!("Playing Next: {}",id).as_bytes().to_vec(), false))
        },
        Commands::PlayPause => {
            let script = r#"
                const mk = MusicKit.getInstance();
                return mk ? (mk.isPlaying || mk.playbackState === 2) : false;
            "#;

            let eval_result = driver.execute(script, vec![])
                .await
                .map_err(|e| e.to_string())?;

            // Extracts the boolean from the returned JSON payload
            let is_playing = eval_result.json().as_bool().unwrap_or(false);

            if is_playing {
                if let Err(e) = driver.pause().await {
                    return Err(e.to_string());
                };

                return Ok(("Pause".to_string().as_bytes().to_vec(), false))
            } else {
                if let Err(e) = driver.play().await {
                    return Err(e.to_string());
                };

                return Ok(("Play".to_string().as_bytes().to_vec(), false))
            }
        },
        Commands::Next  => {
            if let Err(e) = driver.next_track().await {
                    return Err(e.to_string());
                }

            Ok(("Next Track".to_string().as_bytes().to_vec(), false))
        },
        Commands::Prev  => {
            if let Err(e) = driver.previous_track().await {
                return Err(e.to_string());
            }

            Ok(("Previous Track".to_string().as_bytes().to_vec(), false))
        },
        Commands::Shuffle => {
            let mode = driver.shuffle_toggle().await
                    .map_err(|e| e.to_string())?;

                if mode == 1{
                    return Ok(("Shuffling On".to_string().as_bytes().to_vec(), false));
                }
            Ok(("Shuffling Off".to_string().as_bytes().to_vec(), false))
        },
        Commands::ClearQueue => {
            driver.clear_queue().await?;

            Ok(("Queue Cleared".to_string().as_bytes().to_vec(), false))
        },
        Commands::LsQueue => {
            let queue = driver.list_queue().await?;

            let formatted_str:String = queue.iter()
            .map(|track| format!("{}",track.pretty_display()))
            .collect();
            
            Ok((formatted_str.as_bytes().to_vec(), false))
        },
        Commands::Volume { args } => {
            if let Some(v) = args{
                if v > 100 {
                    return Err("Value must be between 0 - 100".to_string());
                }
            }

            let volume = driver.volume(args).await?;

            if args.is_some() {
                Ok((format!("Volume set to: {}", volume).as_bytes().to_vec(), false))
            } else {
                Ok((format!("Volume: {}", volume).as_bytes().to_vec(), false))
            }
        },
        Commands::DisplaModeDaemon => {
            Ok((vec![0u8; 1], true))
        },
        _=> {
            Err("Invalid Command".to_string())
        }
        }
    }
}
