use thirtyfour::WebDriver;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt};
use wincode_derive::{SchemaRead, SchemaWrite};

use crate::browser_interface::MusicDriver;

#[derive(SchemaRead, SchemaWrite)]
pub enum DaemonCommands {
    GetCurrentTrackMeta,
    GetNextTrackMeta,
    PlayToggle,
    Skip,
    Prev,
    VolUp,
    VolDown
}

impl DaemonCommands {
    async fn execute(&self ,driver: &WebDriver) -> Result<Vec<u8>, String> {
        match self {
            DaemonCommands::GetCurrentTrackMeta => {
                let track: Option<crate::browser_interface::Track> = driver.get_current_track().await?;

                if track.is_none() {return Ok(vec![0u8])}


                let bytes: Vec<u8> = wincode::serialize(&track)
                    .map_err(|e| format!("Failed serializing track data: {}", e))?;

                return Ok(bytes)

            },
            DaemonCommands::GetNextTrackMeta => {
                let track_queue = driver.list_queue().await?;

                let track = track_queue.get(0);

                if track.is_none() {return Ok(vec![0u8])}

                let bytes = wincode::serialize(&track)
                    .map_err(|e| format!("Failed serializing track data: {}", e))?;

                return Ok(bytes)
            },
            DaemonCommands::PlayToggle => {
                let script = r#"
                const mk = MusicKit.getInstance();
                return mk ? (mk.isPlaying || mk.playbackState === 2) : false;
            "#;

            let eval_result = driver.execute(script, vec![])
                .await
                .map_err(|e| e.to_string())?;

            let is_playing = eval_result.json().as_bool().unwrap_or(false);

            if is_playing {
                if let Err(e) = driver.pause().await {
                    return Err(e.to_string());
                };

                return Ok(vec![0u8])
            } else {
                if let Err(e) = driver.play().await {
                    return Err(e.to_string());
                };

                return Ok(vec![0u8])
            }
            },
            DaemonCommands::Skip => {
                driver.next_track().await?;
                return Ok(vec![0u8])
            },
            DaemonCommands::Prev => {
                driver.previous_track().await?;
                return Ok(vec![0u8])
            },
            DaemonCommands::VolUp => {
                let vol: u32 = driver.volume(None).await? as u32;
                let new_vol = if vol >= 95 {
                    100
                } else {
                    vol + 5
                };

                driver.volume(Some(new_vol)).await?;

                return Ok(vec![0u8])
            },
            DaemonCommands::VolDown => {
                let vol: u32 = driver.volume(None).await? as u32;
                let new_vol = if vol <= 5 {
                    0
                } else {
                    vol - 5
                };

                driver.volume(Some(new_vol)).await?;

                return Ok(vec![0u8])
            }
        }
    }
}




pub async fn handle_persistent_connection(stream:&mut UnixStream, driver: &WebDriver) -> Result<Vec<u8>, String> {
    let mut len_buf = [0u8; 4];

    stream.read_exact(&mut len_buf).await
        .map_err(|e| format!("Failed reading length prefix: {}", e))?;

    let len = u32::from_le_bytes(len_buf) as usize;

    if len <= 0 {
        return Err("Invalid empty command".to_string());
    }

    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await
        .map_err(|e| format!("Failed reading payload: {}", e))?;

    let command: DaemonCommands = wincode::deserialize(&payload)
        .map_err(|e| format!("Failed Deserializing payload from bytes: {}", e))?;

    let res = command.execute(driver).await?;
    
    Ok(res)
}