use crate::DaemonResponse;
use crate::commands::PlayerCommand;
use thirtyfour::WebDriver;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, BufReader};


pub async fn handle_client(stream: &mut UnixStream, driver: &WebDriver) -> Result<Vec<u8>, String> {
    let mut reader: BufReader<&mut UnixStream> = BufReader::new(stream);

    let mut len_buf = [0u8; 4];

    reader.read_exact(&mut len_buf).await
        .map_err(|e| format!("Failed reading length prefix: {}", e))?;

    let len = u32::from_le_bytes(len_buf) as usize;

    if len <= 0 {
        return Err("Invalid empty command".to_string());
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await
        .map_err(|e| format!("Failed reading payload: {}", e))?;

    let command: PlayerCommand = wincode::deserialize(&payload)
        .map_err(|e| format!("Failed Deserializing payload from bytes: {}", e))?;
    let daemon_res = match command.execute_daemon(driver).await{
        Ok(r) => (DaemonResponse::Success(r.0), r.1),
        Err(e) => (DaemonResponse::Error(e.as_bytes().to_vec()), false)
    };
    let bytes:Vec<u8>  = wincode::serialize(&daemon_res).map_err(|e| format!("Failed Serializing Daemon response: {}",e))?;

    Ok(bytes)
} 