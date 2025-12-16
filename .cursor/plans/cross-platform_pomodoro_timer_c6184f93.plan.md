---
name: Cross-platform Pomodoro Timer
overview: Build a cross-platform pomodoro timer using Tauri (Rust + web frontend) that enforces breaks using a full-screen always-on-top overlay plus OS-level input blocking (best-effort by platform). Breaks normally can’t be skipped; in emergencies the user can bypass a break, and the remaining break time becomes break debt that is added to the next break. Configurable durations and emergency-skip limits.
todos:
  - id: setup-tauri
    content: Initialize Tauri project structure with Rust backend and web frontend
    status: pending
  - id: timer-engine
    content: Implement timer state machine and countdown logic in Rust
    status: pending
    dependencies:
      - setup-tauri
  - id: config-system
    content: Create configuration management system with TOML file storage
    status: pending
    dependencies:
      - setup-tauri
  - id: input-blocking
    content: Implement break overlay + platform-specific input blocking (Windows/macOS/X11) with Wayland fallback behavior
    status: pending
    dependencies:
      - setup-tauri
  - id: skip-logic
    content: Implement emergency skip + break-debt accounting + emergency-skip limit enforcement
    status: pending
    dependencies:
      - timer-engine
      - input-blocking
  - id: frontend-ui
    content: Build frontend UI with timer display, settings panel, and controls
    status: pending
    dependencies:
      - timer-engine
      - config-system
  - id: tauri-commands
    content: Expose Rust functions as Tauri commands for frontend communication
    status: pending
    dependencies:
      - timer-engine
      - config-system
      - input-blocking
  - id: notifications
    content: Add notifications for phase transitions and break period warnings
    status: pending
    dependencies:
      - frontend-ui
  - id: testing
    content: Test on all three platforms and verify overlay + input blocking works correctly
    status: pending
    dependencies:
      - skip-logic
      - frontend-ui
---

# Cross-platform Pomodoro Timer Implementation Plan

## Architecture Overview

**Tech Stack:** Tauri (Rust backend + web frontend)

- **Backend:** Rust for timer logic, break overlay + input blocking integration, config management
- **Frontend:** Web UI (React/Vue/vanilla JS) for settings and timer display
- **Platform Support:** Linux, Windows, macOS

## Core Components

### 1. Timer Engine (`src-tauri/src/timer.rs`)

- Pomodoro cycle state machine (Work → Break → Work → Long Break)
- Timer countdown logic
- Session tracking (current session number, sessions until long break)
- Break debt accumulation (remaining break time added to next break after emergency skip)
- State persistence

### 2. BreakShield Overlay + Input Blocking (`src-tauri/src/input_blocker.rs`)

During breaks, the app shows a full-screen, borderless, always-on-top overlay and blocks input so the computer effectively “does nothing”.

Platform strategy:

- **Windows:** low-level keyboard/mouse hooks (block events) while break is active.
- **macOS:** event taps (block keyboard/mouse events) while break is active.
- **Linux (X11):** grab keyboard/mouse (XGrabKeyboard/XGrabPointer) while break is active.
- **Linux (Wayland):** true global input blocking is generally not possible from an app; fallback is overlay-only + clear warning that full blocking requires an X11 session.

Emergency skip is handled by the overlay (not by system lock/unlock) and is intentionally frictional to discourage casual bypass.

### 4. Configuration Management (`src-tauri/src/config.rs`)

- Pomodoro duration (default: 25 min)
- Break duration (default: 5 min)
- Long break duration (default: 15 min)
- Sessions before long break (default: 4)
- Emergency skip limit (default: 3 per day)
- Optional break-debt cap (default: 60 min)
- Config file: `~/.config/pomohardo/config.toml` (Linux/Mac) or `%APPDATA%/pomohardo/config.toml` (Windows)

### 5. Frontend UI (`src/`)

**Design: Dark theme, minimalist interface**

- **Top Navigation Tabs:**
  - "Timer" tab (selected by default)
  - "Stats" tab (for viewing statistics)

- **Main Timer Display (Timer tab):**
  - Large circular progress indicator:
    - Dark grey arc for elapsed time
    - Bright white arc for remaining time (clockwise from top)
    - Smooth animation as time counts down
  - Phase label displayed inside circle (e.g., "Pomodoro", "Break", "Long Break")
  - Large digital countdown display (MM:SS format) inside circle
  - Centered layout

- **Controls (bottom center):**
  - Pause/Resume button (two vertical bars icon)
  - Skip button (forward arrow with bar icon) - **skip work only** (starts break early). **Never available during breaks.**

- **Break Overlay (during breaks):**
  - Same circular timer UI, but in full-screen always-on-top overlay mode
  - Blocks keyboard/mouse input (best-effort by platform)
  - Emergency skip is **not** a simple button. Use a deliberate sequence:
    - Hold an “arm” key chord for N seconds (default 4s), e.g. `Ctrl+Alt+Shift+E`
    - Then type a short confirm word (default: `SKIP`) and confirm
    - Only then the break can be bypassed

- **Settings Panel:**
  - Accessible via menu or separate settings view
  - Configurable durations (work, break, long break)
  - Sessions before long break setting
  - Emergency skips allowed per day
  - Optional break-debt cap
  - Linux behavior note (Wayland vs X11) if full input blocking is desired

- **Notifications:**
  - System notification when break starts (before overlay)
  - Warning notification if emergency skip limit is reached

- **System tray icon** (optional, for background operation)

### 6. Tauri Commands (`src-tauri/src/main.rs`)

Rust functions exposed to frontend:

- `start_timer()`
- `pause_timer()` / `resume_timer()`
- `skip_work()` - ends work early and starts break (for skip button)
- `arm_emergency_skip()` / `confirm_emergency_skip(confirm_word)` - only valid during break, respects per-day limit, adds remaining break to break debt
- `get_timer_state()` → current phase, time remaining, session count, elapsed time (for progress calculation)
- `get_config()` → current configuration
- `update_config()` → update settings
- `get_break_debt()` → accumulated break debt (seconds)

## Implementation Flow

### Timer State Machine

```
Work → Break → Work → Break → Work → Break → Work → Long Break → (repeat)
```

### Break Period Flow

1. Timer reaches end of work period
2. Show notification: "Break time!"
3. Activate BreakShield overlay + input blocking
4. Break duration is computed as `base_break + break_debt` (debt applies to the next break, including long breaks)
5. If user triggers emergency skip during break (rare, intentionally hard):

   - If daily emergency skip limit reached: deny and keep BreakShield active
   - Otherwise:
     - Require hold-to-arm key chord + confirm word
     - Add remaining break time to `break_debt_seconds`
     - Record emergency skip event (for stats)
     - Exit BreakShield and proceed to next work period

6. If break completes normally:

   - Clear `break_debt_seconds`
   - Proceed to next work period

## File Structure

```
pomohardo/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # Tauri entry, command handlers
│   │   ├── timer.rs              # Timer engine
│   │   ├── input_blocker.rs      # BreakShield overlay + input blocking
│   │   ├── config.rs             # Config management
│   │   └── platform/             # Platform-specific implementations
│   │       ├── linux.rs
│   │       ├── windows.rs
│   │       └── macos.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # Frontend
│   ├── index.html
│   ├── main.js (or main.tsx)
│   ├── components/
│   │   ├── TimerTab.jsx          # Main timer view with circular progress
│   │   ├── StatsTab.jsx          # Statistics view
│   │   ├── CircularTimer.jsx     # Circular progress indicator component
│   │   ├── Controls.jsx          # Pause/Skip buttons
│   │   ├── BreakOverlay.jsx      # Full-screen break overlay (UI + emergency action)
│   │   ├── SettingsPanel.jsx     # Settings modal/panel
│   │   └── NavigationTabs.jsx    # Timer/Stats tab navigation
│   └── styles.css                # Dark theme styles
├── package.json
└── README.md
```

## Key Dependencies

**Rust (Cargo.toml):**

- `tauri` - Framework
- `serde`, `toml` - Config serialization
- `tokio` - Async runtime for timers/background tasks
- `notify` - File system events (for config changes)
- Platform-specific: `x11`/`x11rb` (Linux X11 grab), `windows` crate (Windows hooks), `core-graphics`/`core-foundation` (macOS event taps)

**Frontend:**

- Tauri API client (`@tauri-apps/api`)
- UI framework: Vanilla JS with CSS or lightweight React/Vue for component structure
- SVG/Canvas for circular progress indicator animation
- CSS animations for smooth countdown transitions

## Configuration Schema

```toml
[pomodoro]
work_duration_minutes = 25
break_duration_minutes = 5
long_break_duration_minutes = 15
sessions_before_long_break = 4
emergency_skips_per_day = 3
break_debt_cap_minutes = 60
emergency_arm_chord = "Ctrl+Alt+Shift+E"
emergency_arm_hold_seconds = 4
emergency_confirm_word = "SKIP"
```

## Testing Considerations

- Unit tests for timer logic
- Integration tests for config management
- Manual testing on each platform for input blocking behavior (Windows/macOS/X11) + Wayland fallback warning
- Test emergency skip scenarios (break overlay emergency action)

## Challenges & Solutions

1. **Linux Wayland limitations:** full input blocking is generally not available; provide overlay-only fallback + clear warning + recommend X11 session for full enforcement.
2. **Cross-platform input blocking:** abstract behind a simple trait and implement per platform (Windows hooks / macOS event taps / X11 grabs).
3. **Time correctness:** use monotonic time for countdown (handles sleep/time changes correctly).
4. **State persistence:** save timer state on pause/stop, restore on app restart.
5. **Background operation:** system tray icon, minimize to tray option.