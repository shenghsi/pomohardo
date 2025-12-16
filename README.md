# Pomohardo

A cross-platform pomodoro timer that strictly enforces breaks to encourage healthy work habits.

## Features

- **Strict Break Enforcement**: Uses full-screen overlay + input blocking during breaks
- **Break Debt System**: Skipped break time is added to your next break
- **Emergency Skip**: Limited emergency skips per day with friction (hold chord + confirm)
- **Configurable**: Customize work/break durations, sessions before long break, and more
- **Cross-platform**: Works on Linux, Windows, and macOS

## Build from Source

### Prerequisites

- Node.js and npm
- Rust and Cargo (installed via rustup)

### Platform-Specific Dependencies

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libx11-dev libxtst-dev libxcb1-dev webkit2gtk-4.1-dev \
  build-essential curl libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Windows

- Windows SDK
- Microsoft Visual C++ Build Tools

#### macOS

- Xcode Command Line Tools: `xcode-select --install`

### Installation

```bash
# Install Node dependencies
npm install

# Development mode
npm run dev

# Build for production
npm run build
```

## Configuration

Configuration is stored in:
- **Linux/macOS**: `~/.config/pomohardo/config.toml`
- **Windows**: `%APPDATA%/pomohardo/config.toml`

Default settings:
- Work duration: 25 minutes
- Break duration: 5 minutes
- Long break duration: 15 minutes
- Sessions before long break: 4
- Emergency skips per day: 3
- Break debt cap: 60 minutes

## Usage

1. Click play to start a pomodoro session
2. Work until the timer ends
3. Take a break when the overlay appears
4. In rare emergencies, use the emergency skip (limited per day)
5. Skipped break time accumulates as "break debt" added to your next break

## Platform Notes

### Linux

- **X11**: Full input blocking supported
- **Wayland**: Overlay only (global input blocking not possible without compositor support)

For best enforcement, use an X11 session.

### Windows/macOS

Full input blocking is supported via low-level hooks/event taps.

## License

GPL-3.0

## Inspired By

[GNOME Pomodoro](https://github.com/gnome-pomodoro/gnome-pomodoro) - but with stricter break enforcement.

