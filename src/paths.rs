use std::path::PathBuf;
use std::sync::LazyLock;

pub static BROWSER_PROFILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~/"))
        .join(".config/amix/browser-profile");

    // Automatically create the directory if it doesn't exist
    std::fs::create_dir_all(&path)
        .expect("Failed to create browser profile directory");

    path
});
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut dir = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~/"))
        .join(".config/amix");
    std::fs::create_dir_all(&dir)
        .expect("Failed to create config directory");
    dir.push("config.toml");
    dir
});

pub static DAEMON_ERR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("amix_daemon.err")
});
pub static DAEMON_OUT_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("amix_daemon.out")
});

pub static DAEMON_SOCKET_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("amix_daemon.sock")
});

pub static DAEMON_PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("amix_daemon.pid")
});

pub static GECKO_PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("amix_gecko.pid")
});

pub static BROWSER_PID_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::temp_dir().join("amix_browser.pid")
});

