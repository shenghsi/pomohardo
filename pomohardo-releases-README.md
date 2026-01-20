# Pomohardo - Downloads

A cross-platform Pomodoro timer that enforces breaks to help you maintain healthy work habits.

## Download Latest Version

Choose the installer for your operating system:

### Windows
- **Recommended**: `Pomohardo_x.x.x_x64-setup.exe` (NSIS installer)
- **Alternative**: `Pomohardo_x.x.x_x64_en-US.msi` (MSI installer)

### macOS
- **Intel Macs**: `Pomohardo_x.x.x_x64.dmg`
- **Apple Silicon (M1/M2/M3)**: `Pomohardo_x.x.x_aarch64.dmg`

### Linux
- **Debian/Ubuntu**: `pomohardo_x.x.x_amd64.deb`

---

## Installation Instructions

### Windows

1. Download the `.exe` or `.msi` file
2. Double-click to run the installer
3. Follow the installation wizard
4. Launch Pomohardo from the Start Menu

**Note**: Windows may show a SmartScreen warning because the app is not signed with a commercial certificate. Click "More info" → "Run anyway" to proceed.

### macOS

1. Download the appropriate `.dmg` file for your Mac
2. Open the `.dmg` file
3. Drag Pomohardo to your Applications folder
4. **Important**: Remove the quarantine attribute (see below)

#### Removing macOS Quarantine

macOS Gatekeeper will block unsigned apps. To allow Pomohardo to run:

**Option 1: Using Terminal (Recommended)**
```bash
xattr -dr com.apple.quarantine /Applications/Pomohardo.app
```

**Option 2: Using System Settings**
1. Try to open Pomohardo (it will be blocked)
2. Go to System Settings → Privacy & Security
3. Scroll down to find "Pomohardo was blocked"
4. Click "Open Anyway"
5. Confirm by clicking "Open"

**Why is this needed?**
Pomohardo is not yet signed with an Apple Developer certificate.

### Linux (Debian/Ubuntu)

**Option 1: Using GUI**
1. Download the `.deb` file
2. Double-click to open with Software Install
3. Click "Install"

**Option 2: Using Terminal**
```bash
sudo dpkg -i pomohardo_x.x.x_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
```

**Launch the app:**
```bash
pomohardo
```

Or find it in your application menu.

---

## Auto-Updates

Pomohardo includes automatic update checking. When a new version is available:

1. You'll see a notification dialog
2. Click "Yes" to download and install
3. The app will restart with the new version

Updates are cryptographically signed for security.

---

## Troubleshooting

### macOS: "App is damaged and can't be opened"

This happens when the quarantine attribute is still set. Run:
```bash
xattr -dr com.apple.quarantine /Applications/Pomohardo.app
```

### macOS: App won't start after update

Remove quarantine again after updates:
```bash
xattr -dr com.apple.quarantine /Applications/Pomohardo.app
```

### Linux: Missing dependencies

If the app won't start, install required libraries:
```bash
sudo apt-get install libwebkit2gtk-4.1-0 libayatana-appindicator3-1
```

### Windows: Antivirus blocking the app

Some antivirus software may flag unsigned executables. You can:
1. Add an exception for Pomohardo in your antivirus settings
2. Verify the app is safe by checking the source code repository

---

## Features

- ⏱️ Customizable work and break durations
- 🔒 Enforced breaks with input blocking
- 🚨 Emergency skip with confirmation (limited per day)
- 📊 Session tracking and statistics
- 🌙 Break debt tracking for skipped breaks
- 🔄 Automatic updates
- 🖥️ System tray integration
- 🚀 Auto-start on login (optional)

---

## Support

- **Issues**: Report bugs or request features on the main repository
- **Updates**: This repository is automatically updated with new releases

---

## Privacy

Pomohardo runs entirely on your local machine. No data is collected or sent to any servers.

---

## License

See the main repository for license information.
