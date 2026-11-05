use thirtyfour::prelude::*;
use std::{path::Path, time::Duration, fs};
use tokio::time::sleep;
use std::process::{Command, Child, Stdio};
use crate::{Config, paths::{BROWSER_PID_PATH, BROWSER_PROFILE_PATH, GECKO_PID_PATH}};
pub struct BrowserState {
    pub driver: WebDriver,
    pub geckodriver_process: Child,
}
fn inject_performance_user_prefs() -> Result<(), String>{
    let config = Config::read_config()?;
    let user_js_path = BROWSER_PROFILE_PATH.join("user.js");

    // The mandatory starvation overrides
    let hardening_prefs = format!(r#"
// --- DYNAMIC DAEMON RESOURCE STARVATION OVERRIDES ---
user_pref("layout.frame_rate", {});
user_pref("toolkit.cosmeticAnimations.enabled", {});
user_pref("layers.acceleration.disabled", true);
user_pref("gfx.direct2d.disabled", true);
user_pref("browser.sessionstore.resume_from_crash", false);
user_pref("dom.ipc.processCount", 1);
"#, config.frame_rate, config.cosmetic_animations);

    // Append or overwrite the user.js inside the target profile
    fs::write(user_js_path, hardening_prefs)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn remove_performance_user_prefs() -> Result<(), String>{
    let user_js_path = BROWSER_PROFILE_PATH.join("user.js");

    // The mandatory starvation overrides
    let hardening_prefs = r#"
// --- DYNAMIC DAEMON RESOURCE STARVATION OVERRIDES ---
user_pref("layout.frame_rate", 60);
user_pref("toolkit.cosmeticAnimations.enabled", true);
user_pref("layers.acceleration.disabled", false);
user_pref("gfx.direct2d.disabled", false);
user_pref("browser.sessionstore.resume_from_crash", true);
user_pref("dom.ipc.processCount", 1);
"#;

    // Append or overwrite the user.js inside the target profile
    fs::write(user_js_path, hardening_prefs)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}



pub async fn init_browser(head: bool) -> Result<BrowserState, String> {
let lock_file = BROWSER_PROFILE_PATH.join("lock");
if lock_file.exists() {
    if let Err(e) = fs::remove_file(&lock_file) {
        eprintln!("Could not remove stale profile lock: {}", e);
    } else {
        println!("Removed stale Firefox profile lock.");
    }
}

    if Path::new(&*GECKO_PID_PATH).exists() {
        if let Ok(pid_str) = fs::read_to_string(&*GECKO_PID_PATH) {
            match pid_str.trim().parse::<i32>() {
                Ok(pid) => {
                    unsafe { libc::kill(pid, libc::SIGTERM); }
                }
                Err(_) => {
                    eprintln!("Failed Parsing GECKO_DRIVER pid");
                }
            }
        }

        if let Err(e) = fs::remove_file(&*GECKO_PID_PATH) {
            eprintln!("Could not remove stale GECKO_DRIVER pid: {}", e);
        }
    }

    if Path::new(&*BROWSER_PID_PATH).exists() {
        if let Ok(pid_str) = fs::read_to_string(&*BROWSER_PID_PATH) {
            match pid_str.trim().parse::<i32>() {
                Ok(pid) => {
                    unsafe { libc::kill(pid, libc::SIGTERM); }
                }
                Err(_) => {
                    eprintln!("Failed Parsing BROWSER pid");
                }
            }
        }

        if let Err(e) = fs::remove_file(&*BROWSER_PID_PATH) {
            eprintln!("Could not remove stale BROWSER pid: {}", e);
        }
    }

    let temp_listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find a free port: {}", e))?;
    
    let dynamic_port = temp_listener.local_addr()
        .map_err(|e| format!("Failed to read local port address: {}", e))?
        .port();
    drop(temp_listener);

    if head {
        remove_performance_user_prefs()?;
    } else {
        inject_performance_user_prefs()?;
    }


    let geckodriver_process = if head {
        Command::new("geckodriver")
        .arg("--port")
        .arg(dynamic_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start geckodriver: {}", e))?
    } else {
        let config = Config::read_config()?;
        Command::new("geckodriver")
        .arg("--port")
        .arg(dynamic_port.to_string())
        .env("MOZ_HEADLESS_WIDTH", config.width.to_string())
        .env("MOZ_HEADLESS_HEIGHT", config.height.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start geckodriver: {}", e))?
    };
    let child_pid = geckodriver_process.id();
    if let Err(e) = fs::write(&*GECKO_PID_PATH, child_pid.to_string()) {
        eprintln!("Failed to write gecko PID file: {}", e);
    }

    sleep(Duration::from_millis(500)).await;


    let mut caps = DesiredCapabilities::firefox();
    
    if !head {
        caps.set_headless().map_err(|e| e.to_string())?;
    }
    
    
    if let Some(profile_path) = BROWSER_PROFILE_PATH.to_str() {
    let abs_path = std::fs::canonicalize(profile_path)
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?
        .to_string_lossy()
        .to_string();

    caps.add_arg("--profile").map_err(|e| e.to_string())?;
    caps.add_arg(&abs_path).map_err(|e| e.to_string())?;
    } else {
        return Err("Profile path returned None".to_string());
    }

    let driver = WebDriver::new(format!("http://127.0.0.1:{}", dynamic_port), caps)
        .await
        .map_err(|e| format!("Error connecting WebDriver to port {}: {}", dynamic_port, e))?;

    Ok(BrowserState { driver, geckodriver_process })
}

pub async fn open_apple_music(driver: &WebDriver) -> Result<(), String> {
    driver.goto("https://music.apple.com")
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}








pub async fn is_logged_in(driver: &WebDriver) -> Result<bool, String> {
    let script = "return document.querySelector('.account-menu > amp-contextual-menu-button:nth-child(1) > button:nth-child(1)') !== null;";

    let eval_result = driver.execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;
    let is_logged_in = eval_result.json().as_bool().unwrap_or(false);

    Ok(is_logged_in)
}

pub async fn apple_music_auth() -> Result<(), String> {
    println!("Checking authentication status...");

    // 1. Attempt headless check; if Apple Music rejects/crashes headless, catch it
    let headless_attempt = async {
        let mut state = init_browser(false).await?;
        open_apple_music(&state.driver).await?;
        let logged_in = is_logged_in(&state.driver).await?;
        state.driver.quit().await.map_err(|e| e.to_string())?;
        let _ = state.geckodriver_process.kill();
        Ok::<bool, String>(logged_in)
    }.await;

    let logged_in = match headless_attempt {
        Ok(true) => {
            println!("User already logged in using saved session.");
            return Ok(());
        }
        _ => {
            println!("Headless check bypassed or session not found. Launching headed browser...");
            false
        }
    };

    if !logged_in {
        // 2. Launch in headed mode for manual login
        let mut state = init_browser(true).await?;
        open_apple_music(&state.driver).await?;

        println!("Please log in manually through the browser window...");

        while !is_logged_in(&state.driver).await? {
            sleep(Duration::from_secs(3)).await;
        }

        println!("Logged in successfully!");

        state.driver.quit().await.map_err(|e| e.to_string())?;
        state.geckodriver_process.kill().map_err(|e| e.to_string())?;
    }

    Ok(())
}