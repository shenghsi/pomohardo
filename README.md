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

#### macOS Installation Note

When installing the macOS app, you may see a Gatekeeper warning: "Apple could not verify this app is free of malware." This is expected for apps distributed outside the App Store without Developer ID signing.

**To open the app (macOS Sequoia 15+):**
1. Go to **System Settings → Privacy & Security**
2. Scroll down to the **Security** section at the bottom
3. Find the message about "Pomohardo" being blocked
4. Click **"Open Anyway"**
5. Confirm by clicking "Open" in the dialog

**Alternative method (if the above doesn't appear):**
1. Open Terminal
2. Run: `./scripts/macos-allow-app.sh /path/to/Pomohardo.app` (or manually: `xattr -dr com.apple.quarantine /path/to/Pomohardo.app`)
3. Then go to System Settings → Privacy & Security and click "Open Anyway"

**Note:** The app requires Accessibility permissions for input blocking. You'll be prompted to grant this permission on first use in System Settings → Privacy & Security → Accessibility.

## License

GPL-3.0

## Inspired By

[GNOME Pomodoro](https://github.com/gnome-pomodoro/gnome-pomodoro) - but with stricter break enforcement.

