
use thirtyfour::WebDriver;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, BufReader};
use crate::Commands;


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

    let command: Commands = wincode::deserialize(&payload)
        .map_err(|e| format!("Failed Deserializing payload from bytes: {}", e))?;
    
    Ok(command.execute_daemon(driver).await?)
} 