# Pomohardo

A cross-platform pomodoro timer that **enforces** breaks — so you actually take them.

Most pomodoro timers let you skip breaks with a single click. Pomohardo doesn't. When break time arrives, a fullscreen overlay takes over and blocks input until your break is done. Your brain will thank you.

## Features

- **Strict break enforcement** — fullscreen overlay + input blocking during breaks
- **Break debt** — skipped time accumulates and is added to your next break
- **Emergency skip** — limited skips per day, requires hold + confirm (for real emergencies only)
- **Auto-updates** — new versions install automatically
- **System tray** — runs quietly in the background
- **Auto-start** — optionally launch on login
- **Configurable** — work/break durations, long break interval, and more
- **Cross-platform** — Linux, Windows, and macOS

## Install

Download the latest release for your platform:

| Platform | File |
|----------|------|
| **Windows** | `Pomohardo_x.x.x_x64-setup.exe` |
| **macOS (Intel)** | `Pomohardo_x.x.x_x64.dmg` |
| **macOS (Apple Silicon)** | `Pomohardo_x.x.x_aarch64.dmg` |
| **Linux (Debian/Ubuntu)** | `pomohardo_x.x.x_amd64.deb` |
| **Linux (Fedora/RHEL)** | `pomohardo-x.x.x-1.x86_64.rpm` |

[Latest release](https://github.com/shenghsi/pomohardo/releases/latest)

### macOS

After installing, remove the quarantine attribute:

```bash
xattr -dr com.apple.quarantine /Applications/Pomohardo.app
```

The app will also prompt for Accessibility permissions (required for input blocking during breaks).

If macOS blocks the app, go to **System Settings → Privacy & Security** and click **"Open Anyway"**.

## Build from Source

Prerequisites: [Node.js](https://nodejs.org/), [Rust](https://rustup.rs/)

```bash
git clone https://github.com/shenghsi/pomohardo.git
cd pomohardo
npm install
npm run build
```

Linux requires additional system packages — see the build guide in [SETUP.md](SETUP.md).

## Configuration

Config file location:
- **Linux/macOS**: `~/.config/pomohardo/config.toml`
- **Windows**: `%APPDATA%/pomohardo/config.toml`

| Setting | Default |
|---------|---------|
| Work duration | 25 min |
| Break duration | 5 min |
| Long break duration | 15 min |
| Sessions before long break | 4 |
| Emergency skips per day | 3 |
| Break debt cap | 60 min |

## How It Works

1. Start a pomodoro session
2. Work until the timer ends
3. The overlay appears — take your break
4. In a real emergency, use the emergency skip (limited per day)
5. Skipped break time carries over as debt into your next break

## Platform Notes

- **Linux (X11)**: Full input blocking supported
- **Linux (Wayland)**: Overlay only — compositor support required for input blocking
- **Windows/macOS**: Full input blocking via low-level hooks/event taps

## License

[AGPL-3.0](LICENSE)

## Inspired By

[GNOME Pomodoro](https://github.com/gnome-pomodoro/gnome-pomodoro) — but with stricter break enforcement.
