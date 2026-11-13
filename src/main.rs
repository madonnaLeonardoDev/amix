#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
use std::fs;
use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::net::Shutdown;
use clap::Parser;
use image::EncodableLayout;
use serde::{Deserialize};

use crate::paths::{DAEMON_SOCKET_PATH, CONFIG_PATH};
use crate::daemon::{DaemonResponse, kill_all, start_daemon};
use crate::display_mode::run_player_loop;
use wincode;
mod daemon;
mod handle_client;
mod paths;
mod browser;
mod browser_interface;
mod daemon_controls;
mod display_mode;
mod lyrics;
mod daemon_persistent;
mod commands;
use crate::commands::{Cli, Commands};

#[derive(Deserialize)]
pub struct Config {
    //Browser Headless Settings
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    pub cosmetic_animations: bool
}
impl Config {
    pub fn read_config() -> Result<Self, String> {
        let file_b = fs::read(&*CONFIG_PATH)
            .map_err(|e| format!("Failed reading config file: {}",e))?;

        let config: Config = toml::from_slice(file_b.as_bytes())
            .map_err(|e| format!("Invalid config: {}", e))?;

        Ok(config)
    }

    pub fn init_config() -> Result<(), String> {
        if !CONFIG_PATH.as_path().exists(){
            let default_config = r#"
            #HEADLESS BROWSER DAEMON CONFIG
            width = 1
            height = 1
            frame_rate = 1
            cosmetic_animations = false
            "#;
            if let Some(path) = CONFIG_PATH.parent() {
                fs::create_dir_all(path)
                    .map_err(|e| e.to_string())?;

                fs::write(&*CONFIG_PATH, default_config)
                    .map_err(|e| e.to_string())?;
            }

        }
        Ok(())
    }
}

fn cmd_to_daemon(cli: Cli) {
            let player_cmd = match cli.command.into_player_cmd(){
                Err(e) => {
                    eprint!("{}", e);
                    return;
                }
                Ok(c) => c
            };
            let cmd_as_bytes= wincode::serialize(&player_cmd).unwrap();
            let payload_len = cmd_as_bytes.len() as u32;
            match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()) {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(&payload_len.to_le_bytes()) {
                        eprintln!("Error writing length prefix to stream: {}", e);
                        return;
                    }

                    if let Err(e) = stream.write_all(&cmd_as_bytes) {
                        eprintln!("Error writing payload to stream: {}", e);
                        return;
                    }

                    if let Err(e) = stream.shutdown(Shutdown::Write) {
                        eprintln!("Error on stream shutdown: {}", e);
                        return;
                    }
                    let mut len_b = [0u8; 4];
                    
                    match stream.read_exact(&mut len_b) {
                        Ok(()) => {},
                        Err(e) => {
                            eprintln!("Error reading len header bytes from stream: {}", e);
                            return;
                        }
                    };
                    let len = u32::from_le_bytes(len_b) as usize;
                    let mut buf = vec![0u8; len];
                    if let Err(e) = stream.read_exact(&mut buf) {
                        eprintln!("Error reading payload bytes from stream: {}", e);
                        return;
                    }
                    let res = match DaemonResponse::get_res(buf){
                        Ok(r)=> r,
                        Err(e) => {
                            eprintln!("Error from daemon: {}", e);
                            return;
                        }
                    };
                    match res {
                        DaemonResponse::Success(b) => {
                            if let Ok(s) = String::from_utf8(b){
                                println!("{}", s);
                            } else {
                                eprintln!("Error turning Success bytes into string")
                            }
                        },
                        DaemonResponse::Error(b) => {
                            if let Ok(s) = String::from_utf8(b){
                                eprintln!("{}", s);
                            } else {
                                eprintln!("Error turning Error bytes into string")
                            }
                        }
                    };
                },
                Err(_) => {
                    println!("Daemon is not running. Start it first using `startd (sd)`.");
                }
            }
            
}

fn main() {
    if let Err(e) = Config::init_config() {
        println!("{}", e);
    }
    let cli = Cli::parse();
    match cli.command {
        Commands::Killd => {
            println!("{}",kill_all().unwrap());
        },
        Commands::Startd { head } => {
            if let Err(e) = start_daemon(head) {
                        eprintln!("{}", e);
                        return;
            }
        },
        Commands::Display{lyrics}=> {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    if let Err(e) = rt.block_on(run_player_loop(lyrics)) {
                        eprintln!("Error in player loop: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to create Tokio runtime: {}", e),
            }
        },
        _=>{
            cmd_to_daemon(cli);
        },
    };
}