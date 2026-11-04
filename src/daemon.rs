use std::fs::{self};
use tokio::io::{AsyncWriteExt};
use wincode_derive::{SchemaRead, SchemaWrite};
use std::path::Path;
use daemonize::Daemonize;
use crate::paths::{DAEMON_ERR_PATH, DAEMON_OUT_PATH, DAEMON_PID_PATH, DAEMON_SOCKET_PATH, GECKO_PID_PATH};
use crate::handle_client::{handle_client};
use crate::browser::{apple_music_auth, init_browser, open_apple_music};
use crate::daemon_persistent::handle_persistent_connection;

#[derive(SchemaWrite, SchemaRead)]
pub enum DaemonResponse {
    Success(Vec<u8>),
    Error(Vec<u8>)
}

impl DaemonResponse {
    pub fn bytes_err(err_msg: String) -> Result<Vec<u8>, String> {
        wincode::serialize(&DaemonResponse::Error(err_msg.as_bytes().to_vec()))
            .map_err(|e| format!("Failed Serializing Daemon Error Res: {}", e))
    }

    pub fn bytes_succ(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        wincode::serialize(&DaemonResponse::Success(bytes))
            .map_err(|e| format!("Failed Serializing Daemon Success Res: {}", e))
    }
    
    pub fn get_res(res: Vec<u8>) -> Result<Self, String> {
        wincode::deserialize(&res)
            .map_err(|e| format!("Failed Deserializing Daemon Res: {}", e))
    }
}

pub fn start_daemon(head: bool) -> Result<(), String> {
    //CHECK AND CLEANUP (SOCK + PID)    

    if Path::new(DAEMON_PID_PATH.as_path()).exists() {
        if let Ok(pid_str) = fs::read_to_string(DAEMON_PID_PATH.as_path()) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                println!("Found existing PID file. Checking if process {} is alive...", pid);

                let is_alive = unsafe { libc::kill(pid as i32, 0) == 0 };
                
                if is_alive {
                    return Err(format!("Daemon is already running (PID {}). Aborting.", pid));
                    
                } else {
                    println!("Found stale PID file (process is dead). Cleaning up...");
                }
            }
        }
        fs::remove_file(&*DAEMON_PID_PATH)
        .map_err(|e| format!("Could not remove file at: {}, {}", &DAEMON_PID_PATH.display(), e))?;
    }

    if Path::new(DAEMON_SOCKET_PATH.as_path()).exists() {
        fs::remove_file(DAEMON_SOCKET_PATH.as_path())
        .map_err(|e| format!("Could not remove .sock file at: {}, {}", &DAEMON_SOCKET_PATH.display(), e))?;
    }

    //SETUP deamonize

    let stdout = fs::File::create(DAEMON_OUT_PATH.as_path())
        .map_err(|e| format!("Failed to create stdout log: {}", e))?;
    let stderr = fs::File::create(DAEMON_ERR_PATH.as_path())
        .map_err(|e| format!("Failed to create stderr log: {}", e))?;

    let daemonize = Daemonize::new()
        .pid_file(&*DAEMON_PID_PATH)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    println!("Starting applemusic-daemon..."); 

    if let Err(e) = daemonize.start() {
        return Err(format!("Error in starting daemon: {}", e));
    }
    
    // async tokio runtime
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to boot async runtime: {}", e))?;

    rt.block_on(async {
        // Auth check
        match apple_music_auth().await {
            Err(e) => {
                eprintln!("Auth error: {}", e);
                return Err("Error, cant start apple music auth".to_string());
            },
            Ok(_) => {
                let state = init_browser(head).await?;
                open_apple_music(&state.driver).await?;
                // Bind Tokio async Unix socket listener
                let listener = tokio::net::UnixListener::bind(&*DAEMON_SOCKET_PATH)
                    .map_err(|e| format!("Error in binding socket listener: {}", e))?;

                let state = std::sync::Arc::new(state);
                loop {
                    match listener.accept().await {
                        Ok((mut stream, _)) => {
                            let state = state.clone();

tokio::spawn(async move {
    // Helper closure to write an error DaemonResponse safely before exiting
    async fn send_error(stream: &mut tokio::net::UnixStream, msg: String) {
        if let Ok(e_bytes) = DaemonResponse::bytes_err(msg) {
            let len_b = (e_bytes.len() as u32).to_le_bytes();
            let _ = stream.write_all(&len_b).await;
            let _ = stream.write_all(&e_bytes).await;
        }
    }

    match handle_client(&mut stream, &state.driver).await {
        Ok(r) => {
            let payload: Result<(Vec<u8>, bool), String> = match wincode::deserialize(&r) {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Error deserializing payload flags: {}", e);
                    eprintln!("{}", err_msg);
                    send_error(&mut stream, err_msg).await;
                    return;
                }
            };

            let (initial_res_bytes, is_persistent) = match payload {
                Ok((data, persistent)) => {
                    let bytes = match DaemonResponse::bytes_succ(data) {
                        Ok(b) => b,
                        Err(e) => {
                            let err_msg = format!("Response building error: {}", e);
                            eprintln!("{}", err_msg);
                            match DaemonResponse::bytes_err(err_msg) {
                                Ok(b) => b,
                                Err(_) => return,
                            }
                        }
                    };
                    (bytes, persistent)
                }
                Err(err_str) => {
                    let bytes = match DaemonResponse::bytes_err(err_str) {
                        Ok(b) => b,
                        Err(err) => {
                            let err_msg = format!("Error formatting error response: {}", err);
                            eprintln!("{}", err_msg);
                            return;
                        }
                    };
                    (bytes, false)
                }
            };

            let len_b = (initial_res_bytes.len() as u32).to_le_bytes();
            if stream.write_all(&len_b).await.is_err() || stream.write_all(&initial_res_bytes).await.is_err() {
                eprintln!("Failed to write initial response to client");
                return;
            }

            if is_persistent {
                loop {
                    match handle_persistent_connection(&mut stream, &state.driver).await {
                        Ok(r) => {
                            let res = match DaemonResponse::bytes_succ(r) {
                                Ok(bytes) => bytes,
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break; 
                                }
                            };

                            let len_b = (res.len() as u32).to_le_bytes();
                            if stream.write_all(&len_b).await.is_err() || stream.write_all(&res).await.is_err() {
                                break; 
                            }
                        },
                        Err(e) => {
                            let res = match DaemonResponse::bytes_err(e) {
                                Ok(bytes) => bytes,
                                Err(err) => {
                                    eprintln!("{}", err);
                                    break;
                                }
                            };

                            let len_b = (res.len() as u32).to_le_bytes();
                            if stream.write_all(&len_b).await.is_err() || stream.write_all(&res).await.is_err() {
                                break; 
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            send_error(&mut stream, e).await;
        },
    }
    
    if let Err(e) = stream.shutdown().await {
        eprintln!("Error shutting down stream: {}", e);
    }
});
                        }
                        Err(e) => {
                            eprintln!("Connection failed: {}", e);
                        }
                    }
                }
            }
        }
    })
}

pub fn kill_all() -> Result<String, String> {
    let _ = kill_pid(&*DAEMON_PID_PATH);
    let _ = kill_pid(&*GECKO_PID_PATH);

    if Path::new(&*DAEMON_SOCKET_PATH).exists() {
        fs::remove_file(&*DAEMON_SOCKET_PATH)
            .map_err(|e| format!("Could not remove socket file at: {}, {}", &DAEMON_SOCKET_PATH.display(), e))?;
    }

    Ok("All Processes killed successfully".to_string())
}

pub fn kill_pid(pid_path: &Path) -> Result<String, String> {
    if !Path::new(pid_path).exists() {
        return Ok(format!("{} is not running", pid_path.display()));
    }

    let pid_str = fs::read_to_string(pid_path)
        .map_err(|e| format!("Could not read pid at: {}, {}", pid_path.display(), e))?;
        
    let pid = pid_str.trim().parse::<i32>()
        .map_err(|e| format!("Could not parse pid, {}", e))?;

    unsafe {
        libc::kill(pid, libc::SIGTERM);
    };

    if Path::new(pid_path).exists() {
        fs::remove_file(pid_path)
            .map_err(|e| format!("Could not remove PID file at: {}, {}", pid_path.display(), e))?;
    }

    Ok(format!("PID: {} killed successfully", pid_path.display()))
}