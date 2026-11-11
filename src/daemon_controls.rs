use thirtyfour::{WebDriver};
use crate::{browser_interface::*, commands::{MediaType, PlayerCommand}};

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

impl PlayerCommand {
    pub async fn execute_daemon(self, driver:&WebDriver) -> Result<(Vec<u8>, bool /* requires persistent connection */), String> {
        match self {
        //WORKING
        PlayerCommand::Search(search_args) => {
            let search_res = search_args.resolve_search(driver).await?;
                    let res = match search_res {
                        SearchResult::Track(track) => {
                            let id = if track.id.is_none() {
                            return Ok(("No track id".as_bytes().to_vec(), false));
                            } else {
                            track.id.clone().unwrap()
                            };

                            (track.pretty_display(), id)
                    },
                    SearchResult::Playlist(playlist) => {
                        let id = if playlist.id.is_none() {
                            return Ok(("No Playlist id".as_bytes().to_vec(), false));
                        } else {
                            playlist.id.clone().unwrap()
                        };

                        (playlist.pretty_display(), id)
                    },
                    SearchResult::Album(album) => {
                        let id = if album.id.is_none() {
                            return Ok(("No Album id".as_bytes().to_vec(), false));
                        } else {
                            album.id.clone().unwrap()
                        };

                        (album.pretty_display(), id)

                    }
                    SearchResult::Artist(artist) => {
                        let id = if artist.id.is_none() {
                            return Ok(("No Artist id".as_bytes().to_vec(), false));
                        } else {
                            artist.id.clone().unwrap()
                        };

                        (artist.pretty_display(), id)
                    }
                };
                Ok((res.0.as_bytes().to_vec(), false))
            }
        //WORKING
        PlayerCommand::Current => {
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
        PlayerCommand::ListPlaylists => {
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
        //WORKING
        PlayerCommand::SetQueue(search_args) => {
            let is_library = search_args.is_target_library;
            let search_res = search_args.resolve_search(driver).await?;
                let (res_type, id) = match search_res {
                    SearchResult::Album(album) => {
                        if album.id.is_none() {
                            return Ok(("No Track found".as_bytes().to_vec(), false))
                        }
                        (MediaType::Album, album.id)  
                    },
                    SearchResult::Artist(artist) => {
                        if artist.id.is_none() {
                            return Ok(("No Track found".as_bytes().to_vec(), false))
                        }
                        (MediaType::Artist, artist.id)  
                    },
                    SearchResult::Playlist(playlist) => {
                        if playlist.id.is_none() {
                            return Ok(("No Track found".as_bytes().to_vec(), false))
                        }
                        (MediaType::Playlist, playlist.id)  
                    },
                    SearchResult::Track(track) => {
                        if track.id.is_none() {
                            return Ok(("No Track found".as_bytes().to_vec(), false))
                        }
                        (MediaType::Song, track.id)  
                    }
                };
                let id = driver.set_queue(&res_type, &id.unwrap(), is_library).await
                .map_err(|e| e.to_string())?;
                
                driver.play().await?;

                return Ok((format!("Playing: {}",id).as_bytes().to_vec(), false))
            }
        //WORKING
        PlayerCommand::PlayStop => {
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
        //WORKING
        PlayerCommand::NextTrack  => {
            if let Err(e) = driver.next_track().await {
                    return Err(e.to_string());
                }

            Ok(("Next Track".to_string().as_bytes().to_vec(), false))
        },
        //WORKING
        PlayerCommand::PrevTrack  => {
            if let Err(e) = driver.previous_track().await {
                return Err(e.to_string());
            }

            Ok(("Previous Track".to_string().as_bytes().to_vec(), false))
        },
        //WORKING
        PlayerCommand::ShuffleToggle => {
            let mode = driver.shuffle_toggle().await
                    .map_err(|e| e.to_string())?;

                if mode == 1{
                    return Ok(("Shuffling On".to_string().as_bytes().to_vec(), false));
                }
            Ok(("Shuffling Off".to_string().as_bytes().to_vec(), false))
        },
        //WORKING
        PlayerCommand::RmQueue(index) => {
            match index {
                None => {
                    driver.clear_queue().await?;
                    Ok(("Queue Cleared".to_string().as_bytes().to_vec(), false))
                },
                Some(i) => {
                    driver.remove_queue(i).await?;
                    Ok((format!("Remove item at index: {}", i).as_bytes().to_vec(), false))
                }
            }
        },
        //WORKING
        PlayerCommand::ListQueueTracks(limit) => {
            let queue = driver.list_queue().await?;
            let mut formatted_str: String = String::new();
            let tracks = match limit {
                Some(l) => queue.into_iter().take(l as usize).collect::<Vec<Track>>(),
                None => queue 
            };
            for track in tracks {
                formatted_str.push_str(&track.pretty_display());
            }
            Ok((formatted_str.as_bytes().to_vec(), false))
        },
        //WORKING
        PlayerCommand::Volume(val) => {
            if let Some(v) = val{
                if v > 100 {
                    return Err("Value must be between 0 - 100".to_string());
                }
            }

            let volume = driver.volume(val).await?;

            if val.is_some() {
                Ok((format!("Volume set to: {}", volume).as_bytes().to_vec(), false))
            } else {
                Ok((format!("Volume: {}", volume).as_bytes().to_vec(), false))
            }
        },
        //WORKING
        PlayerCommand::DisplaModeDaemon => {
            Ok((vec![0u8; 1], true))
        },
        //WORKING
        PlayerCommand::GetNextTrackMeta => {
                let track_queue = driver.list_queue().await?;

                let track = track_queue.get(1);

                if track.is_none() {return Ok((vec![0u8], false))}

                let bytes = wincode::serialize(&track)
                    .map_err(|e| format!("Failed serializing track data: {}", e))?;

                return Ok((bytes, false))
            },
        //WORKING
        PlayerCommand::QueueAdd{search: search_args, is_next} => {
            let search_res = driver.search(&search_args.target, search_args.search_type, search_args.is_target_library, search_args.limit.unwrap_or(1)).await?;
            match search_res {
                SearchResult::Album(item) => {
                    let id = match item.id {
                        Some(id) => id,
                        None => return Err("Cannot get results id".to_string())
                    };
                    driver.queue(&MediaType::Album,&id, is_next).await?;

                    return Ok((format!("Added to queue: {}", id).as_bytes().to_vec(), false));
                },
                SearchResult::Playlist(item) => {
                    let id = match item.id {
                        Some(id) => id,
                        None => return Err("Cannot get results id".to_string())
                    };
                    driver.queue(&MediaType::Playlist,&id, is_next).await?;

                    return Ok((format!("Added to queue: {}", id).as_bytes().to_vec(), false));
                },
                SearchResult::Track(item) => {
                    let id = match item.id {
                        Some(id) => id,
                        None => return Err("Cannot get results id".to_string())
                    };
                    driver.queue(&MediaType::Song,&id, is_next).await?;

                    return Ok((format!("Added to queue: {}", id).as_bytes().to_vec(), false));
                },
                SearchResult::Artist(_) => {
                    return Err("Cannot queue an artist".to_string());
                }
            }
        },
        //WORKING
        PlayerCommand::AutoPlayToggle => {
            let res = driver.autoplay_toggle().await?;
            if res {
                Ok(("Autoplay On".as_bytes().to_vec(), false))
            } else {
                Ok(("Autoplay Off".as_bytes().to_vec(), false))
            }
            
        }
        _=> {
            Err("Invalid Command".to_string())
        }
        }
    }
}
