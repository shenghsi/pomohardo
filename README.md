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
🍅 Pomohardo: The Timer That Actually Makes You Rest
Stop lying to yourself. Start actually taking breaks.

We've all been there. The gentle "ding" of a standard Pomodoro timer goes off. You think, "Just one more minute," or "I'm in the flow, I'll skip this one." Two hours later, your back hurts, your eyes are dry, and your productivity has tanked.

Pomohardo is different. It doesn't just suggest a break—it enforces it.

Why You Need This
Discipline is hard. Alerts are easy to ignore. Pomohardo acts as the strict accountability partner you didn't know you needed. When your work session ends, Pomohardo takes over your screen, blocking input so you have to step away, stretch, and reset.

It isn't just annoying; it's designed to be sustainable. We know emergencies happen, so we built a system that respects your workflow while protecting your health.

🛡️ Key Features
⛔ Strict Enforcement: When it's break time, a full-screen shield appears. On Linux (X11) and Windows/macOS, it blocks mouse and keyboard input. You literally cannot work until the break is over.
💳 Break Debt System: Need to skip a break for a meeting? Fine. But you can't cheat physics. Skipped break time is automatically added to your next break. You owe that rest to your body.
🚨 Emergency Skips (with Friction): We know life happens. You get a limited number of emergency skips per day. To use one, you have to physically hold a difficult key combo (Ctrl+Alt+Shift+E) and type a confirmation code. It adds just enough friction to stop impulsive skipping.
💻 Cross-Platform: Native support for Linux, Windows, and macOS.
🎨 Minimal & Lightweight: Built with Rust and Tauri, it uses minimal resources so your computer stays fast.
How It Works
Focus: work for 25 minutes (customizable).
Freeze: When time is up, the Break Shield engages. Input is blocked.
Recharge: Walk away. Stretch. Drink water. Refill your mana.
Resume: Come back refreshed and ready to crush the next session.
Ready to actually get things done?
Don't let burnout kill your momentum. Let Pomohardo protect your energy.

👉 [Download Pomohardo functionality now] (Link to your release/repo)
- **X11**: Full input blocking supported
- **Wayland**: Overlay only (global input blocking not possible without compositor support)

For best enforcement, use an X11 session.

### Windows/macOS

Full input blocking is supported via low-level hooks/event taps.

#### macOS Installation Note

When installing the macOS app, you may see a Gatekeeper warning: "Apple could not verify this app is free of malware." This is expected for apps distributed outside the App Store without Developer ID signing.

**To open the app:**

1. **Remove quarantine attribute** (recommended first step):
   - Open Terminal
   - Run: `xattr -dr com.apple.quarantine /path/to/Pomohardo.app` (or use `./scripts/macos-allow-app.sh`)
   - Replace `/path/to/Pomohardo.app` with the actual path (e.g., `~/Downloads/Pomohardo.app` or `/Applications/Pomohardo.app`)

2. **Open the app:**
   - If you see a message in **System Settings → Privacy & Security → Security** section, click **"Open Anyway"** and confirm
   - If there's no message in Security settings, try opening the app directly - it should work after removing the quarantine attribute

**Note:** The app requires Accessibility permissions for input blocking. You'll be prompted to grant this permission on first use in System Settings → Privacy & Security → Accessibility.

## License

GPL-3.0

## Inspired By

[GNOME Pomodoro](https://github.com/gnome-pomodoro/gnome-pomodoro) - but with stricter break enforcement.

