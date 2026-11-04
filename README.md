# amix

A lightweight, high-performance Rust command-line client and daemon for Apple Music on Linux. 

`amix` bypasses the slow, heavy browser UI by interacting directly with the **MusicKit V3 API** through a headless Firefox background instance. This approach completely circumvents DRM constraints while stripping away browser overhead and eliminating long event pipelines for instant command response times.

---

<p align="center">
  <img src="assets/display-demo.png" alt="amix Terminal Display with Cover Art and Scrolling Lyrics" width="800">
</p>

---

## Key Features

* **Headless Firefox Backend:** Resolves Apple Music DRM restrictions natively using a minimal browser instance running in the background.
* **Direct MusicKit V3 API Integration:** Skips bloated web UI rendering and complex click pipelines, delivering snappy, low-latency playback control.
* **Terminal Display Mode:** Real-time terminal interface rendering cover art (for supported terminal emulators like Kitty or Sixel) alongside synced, scrolling lyrics.
* **LRCLIB Integration:** Synced lyrics fetched automatically from [lrclib.net](https://lrclib.net).
* **Ultra-Low Resource Footprint:**
  * **Rust TUI / CLI:** ~0% CPU usage and ~2% RAM on standard hardware (~32GB DDR4 / Ryzen 5 5600X).
  * **Daemon Overhead:** Skips all UI rendering, using barely ~0.2% RAM (~65MB).
* **Window Manager Friendly:** Designed to be effortlessly bound to keyboard shortcuts in `i3wm`, `sway`, `hyprland`, `bspwm`, or `xbindkeys`.
* **Full Music Control:** Search, queue manipulation, volume adjustment, track navigation, and real-time playback monitoring.

---

## OS & Platform Support

* **Primary Target:** Developed, optimized, and exclusively tested on **Linux** (Arch Linux / `i3wm`).
* **Other Platforms:** While Rust and Firefox are cross-platform, Linux is the main supported environment. Issues on non-Linux platforms may not be actively addressed.
* **Maintenance Status:** The project is actively maintained with bug fixes, optimizations, and new feature updates planned for future releases.

---

## Requirements

* **Firefox** (installed on your system).
* **Rust Compiler & Cargo** (only if building from source).
* **Image-capable terminal emulator** (optional, required only for terminal cover art rendering, e.g., Kitty, WezTerm, Konsole).

---

## Installation

### Option 1: Precompiled Binary
Download the precompiled binary directly from the release folder inside the repository:
```bash
cp bin/amix /usr/local/bin/

```
---

### Option 2: Build from Source

Ensure you have the Rust toolchain installed, then run:

```
cargo build --release

```
### Getting Detailed Help

To view all available subcommands and global options:

```
applemusic-cli --help
# or
applemusic-cli -h


```

To see specific flags, options, and syntax for any subcommand (e.g., `startd` or `search`):

```
amix startd --help
amix search --help


```

## 🔧 Troubleshooting & Setup Notes

### 1. DRM Constraints on New Firefox Instances

If Firefox fails to play tracks due to missing DRM plugins in a clean headless instance:

1. Open your main Firefox browser and ensure Widevine/DRM content playback is enabled and working on Apple Music.

2. Copy your existing Firefox profile containing the DRM configuration into the app's designated profile directory:

   ```
   rm -r  ~/.config/amix/browser-profile/*
   cp -r ~/.mozilla/firefox/<your-profile-folder>/* ~/.config/amix/browser-profile/
   
   
   ```

### 2. Autoplay Blocked by Firefox

If Firefox blocks automated playback commands upon startup:

1. Start the daemon in headed mode:

   ```
   amix startd -H
   
   
   ```

2. In the opened Firefox window, navigate to Apple Music, open site permissions (click the lock icon in the address bar), and set **Autoplay** to **"Allow Audio and Video"**.

3. Close the window and resume running the daemon headlessly.

## Maintenance & Future Updates

This project is actively maintained. Bug fixes, performance tweaks, and feature improvements will be pushed regularly. Feel free to open issues or submit pull requests on the GitHub repository!