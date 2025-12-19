# Requirements Document

## Introduction

This specification addresses cross-platform compatibility gaps in the Pomohardo application, a Tauri-based pomodoro timer. The application currently has full functionality on Linux (X11) but has placeholder implementations for Windows and macOS input blocking and emergency chord features. Additionally, platform-specific UX conventions (macOS tray behavior) and code quality issues (hardcoded paths) need to be addressed.

## Glossary

- **Pomohardo**: The cross-platform pomodoro timer application built with Tauri
- **Input Blocking**: Feature that prevents keyboard and mouse input during enforced breaks
- **Emergency Chord**: A keyboard shortcut (Ctrl+Alt+Shift+E) that allows users to bypass input blocking in emergencies
- **BreakShield**: The fullscreen overlay window displayed during breaks
- **Tray Icon**: System tray/menu bar icon that shows timer progress and provides quick actions
- **X11**: The windowing system used on most Linux distributions
- **Wayland**: A newer Linux display protocol with limited global input interception capabilities
- **SetWindowsHookEx**: Windows API for installing system-wide keyboard/mouse hooks
- **CGEventTap**: macOS Core Graphics API for intercepting system-wide input events

## Requirements

### Requirement 1

**User Story:** As a Windows user, I want input blocking during breaks, so that I am encouraged to take proper rest periods without being tempted to continue working.

#### Acceptance Criteria

1. WHEN a break starts on Windows THEN the InputBlocker SHALL install low-level keyboard hooks using SetWindowsHookEx with WH_KEYBOARD_LL
2. WHEN a break starts on Windows THEN the InputBlocker SHALL install low-level mouse hooks using SetWindowsHookEx with WH_MOUSE_LL
3. WHEN a break ends on Windows THEN the InputBlocker SHALL remove all installed hooks and release system resources
4. IF hook installation fails on Windows THEN the InputBlocker SHALL return an error message describing the failure reason
5. WHILE input blocking is active on Windows THEN the InputBlocker SHALL block keyboard and mouse events from reaching other applications

### Requirement 2

**User Story:** As a macOS user, I want input blocking during breaks, so that I am encouraged to take proper rest periods without being tempted to continue working.

#### Acceptance Criteria

1. WHEN a break starts on macOS THEN the InputBlocker SHALL create an event tap using CGEventTapCreate with kCGHeadInsertEventTap
2. WHEN a break starts on macOS THEN the InputBlocker SHALL intercept keyboard and mouse events via the event tap
3. WHEN a break ends on macOS THEN the InputBlocker SHALL disable and release the event tap
4. IF event tap creation fails on macOS THEN the InputBlocker SHALL return an error message indicating Accessibility permissions may be required
5. WHILE input blocking is active on macOS THEN the InputBlocker SHALL suppress keyboard and mouse events from reaching other applications

### Requirement 3

**User Story:** As a Windows user, I want an emergency keyboard shortcut to skip breaks, so that I can handle urgent situations without being locked out of my computer.

#### Acceptance Criteria

1. WHEN the emergency chord (Ctrl+Alt+Shift+E) is pressed on Windows THEN the InputBlocker SHALL detect the key combination
2. WHILE input blocking is active on Windows THEN the emergency chord detection SHALL continue to function
3. WHEN emergency_chord_pressed is called on Windows THEN the InputBlocker SHALL return true if the chord is currently pressed, false otherwise

### Requirement 4

**User Story:** As a macOS user, I want an emergency keyboard shortcut to skip breaks, so that I can handle urgent situations without being locked out of my computer.

#### Acceptance Criteria

1. WHEN the emergency chord (Ctrl+Alt+Shift+E) is pressed on macOS THEN the InputBlocker SHALL detect the key combination
2. WHILE input blocking is active on macOS THEN the emergency chord detection SHALL continue to function
3. WHEN emergency_chord_pressed is called on macOS THEN the InputBlocker SHALL return true if the chord is currently pressed, false otherwise

### Requirement 5

**User Story:** As a macOS user, I want the tray icon to follow macOS conventions, so that the application feels native and intuitive on my platform.

#### Acceptance Criteria

1. WHEN the user left-clicks the tray icon on macOS THEN the Tray module SHALL display the context menu
2. WHEN the user left-clicks the tray icon on Windows or Linux THEN the Tray module SHALL toggle the main window visibility
3. WHEN the tray icon is created on macOS THEN the TrayIconBuilder SHALL be configured with show_menu_on_left_click set to true

### Requirement 6

**User Story:** As a developer, I want the icon generation scripts to use cross-platform path handling, so that the scripts work correctly on Windows, macOS, and Linux.

#### Acceptance Criteria

1. WHEN constructing file paths in generate-icons-simple.js THEN the script SHALL use path.join() instead of hardcoded forward slashes
2. WHEN constructing file paths in generate-icons.js THEN the script SHALL use path.join() instead of hardcoded forward slashes
3. WHEN the scripts are executed on Windows THEN the scripts SHALL correctly resolve paths using the native path separator

### Requirement 7

**User Story:** As a developer, I want the Linux instance lock mechanism to be documented, so that future maintainers understand why it exists alongside the Tauri single-instance plugin.

#### Acceptance Criteria

1. WHEN reviewing the acquire_instance_lock function THEN the code SHALL contain documentation comments explaining the purpose of the custom lock
2. WHEN reviewing the documentation THEN the comments SHALL explain the relationship between the custom lock and tauri-plugin-single-instance
