---
name: Cross-Platform Compatibility Fixes
overview: "Fix all cross-platform compatibility issues: implement input blocking and emergency chord for Windows/macOS, make macOS tray behavior platform-specific, investigate Linux instance lock, and fix hardcoded paths in scripts."
todos:
  - id: windows_input_blocking
    content: Implement Windows input blocking using SetWindowsHookEx with WH_KEYBOARD_LL and WH_MOUSE_LL hooks in input_blocker.rs
    status: pending
  - id: macos_input_blocking
    content: Implement macOS input blocking using CGEventTap via core-graphics crate in input_blocker.rs
    status: pending
  - id: windows_emergency_chord
    content: Implement emergency chord detection for Windows using RegisterHotKey or GetAsyncKeyState in input_blocker.rs
    status: pending
  - id: macos_emergency_chord
    content: Implement emergency chord detection for macOS using CGEventTap or Carbon hotkey API in input_blocker.rs
    status: pending
  - id: update_emergency_command
    content: Update emergency_chord_pressed command in main.rs to call platform-specific implementations instead of returning false
    status: pending
    dependencies:
      - windows_emergency_chord
      - macos_emergency_chord
  - id: macos_tray_behavior
    content: "Make tray icon behavior platform-specific: show menu on left-click for macOS, keep toggle behavior for others in tray.rs"
    status: pending
  - id: investigate_linux_lock
    content: Investigate and document why custom Linux instance lock exists alongside tauri-plugin-single-instance in main.rs
    status: pending
  - id: fix_icon_script_paths
    content: Replace hardcoded paths with path.join() in generate-icons-simple.js and generate-icons.js
    status: pending
---

# Cross-Platform Compatibility Fixes

## Overview

This plan addresses all identified cross-platform issues in the Pomohardo codebase, ensuring feature parity across Linux, Windows, and macOS.

## Critical Issues (Feature Gaps)

### 1. Implement Windows Input Blocking

**File:** `src-tauri/src/input_blocker.rs`Currently, `activate_windows()` and `deactivate_windows()` are placeholders. Implement using `SetWindowsHookEx` with `WH_KEYBOARD_LL` and `WH_MOUSE_LL` hooks via the `windows` crate (already in dependencies).**Changes:**

- Add Windows-specific state to `InputBlocker` struct (similar to Linux `x11` field)
- Implement `activate_windows()` using `SetWindowsHookEx` for keyboard and mouse hooks
- Implement `deactivate_windows()` to remove hooks and clean up
- Handle hook callback functions to block input events
- Add proper error handling for hook installation failures

**Dependencies:** `windows` crate features already configured in `Cargo.toml`

### 2. Implement macOS Input Blocking

**File:** `src-tauri/src/input_blocker.rs`Currently, `activate_macos()` and `deactivate_macos()` are placeholders. Implement using `CGEventTap` via the `core-graphics` crate (already in dependencies).**Changes:**

- Add macOS-specific state to `InputBlocker` struct (event tap reference)
- Implement `activate_macos()` using `CGEventTapCreate` with `kCGHeadInsertEventTap`
- Implement `deactivate_windows()` to disable and release event tap
- Handle event tap callback to block keyboard/mouse events
- Add error handling for Accessibility permission requirements
- Consider adding permission check/request logic

**Dependencies:** `core-graphics` crate already configured in `Cargo.toml`

### 3. Implement Emergency Skip Chord for Windows

**File:** `src-tauri/src/input_blocker.rs`Add `emergency_chord_pressed()` implementation for Windows using global hotkey detection.**Changes:**

- Add Windows hotkey registration using `RegisterHotKey` API
- Implement key state checking via `GetAsyncKeyState` or hook callbacks
- Detect Ctrl+Alt+Shift+E combination
- Integrate with existing `emergency_chord_pressed()` method

### 4. Implement Emergency Skip Chord for macOS

**File:** `src-tauri/src/input_blocker.rs`Add `emergency_chord_pressed()` implementation for macOS using global hotkey detection.**Changes:**

- Use `CGEventTap` or Carbon hotkey API to detect chord
- Implement key state checking for Ctrl+Alt+Shift+E
- Integrate with existing `emergency_chord_pressed()` method
- May reuse event tap from input blocking if already active

### 5. Update Emergency Chord Command

**File:** `src-tauri/src/main.rs`Remove the `#[cfg(not(target_os = "linux"))] `block that returns `Ok(false)`, allowing platform-specific implementations to be called.**Changes:**

- Modify `emergency_chord_pressed()` command to call platform-specific implementations
- Ensure all platforms can use the emergency chord feature

## High Priority Issues (UX)

### 6. Make macOS Tray Icon Behavior Platform-Specific

**File:** `src-tauri/src/tray.rs`Make tray icon behavior follow macOS conventions: menu opens on left-click instead of toggling window.**Changes:**

- Use conditional compilation to set `show_menu_on_left_click(true)` on macOS
- Keep `show_menu_on_left_click(false)` on other platforms
- Remove or conditionally disable the left-click window toggle handler on macOS
- Ensure right-click still works consistently across platforms

**Lines to modify:** Around line 183 in `tray.rs`

## Medium Priority Issues (Code Quality)

### 7. Investigate and Document Linux Instance Lock

**File:** `src-tauri/src/main.rs`The custom Linux instance lock (`acquire_instance_lock()`) exists alongside `tauri-plugin-single-instance`. Investigate why it was added and document the reasoning.**Changes:**

- Review git history or comments to understand why custom lock was added
- Test if `tauri-plugin-single-instance` alone is sufficient on Linux
- Add documentation comment explaining the dual-lock mechanism
- If redundant, consider removing it (but keep for now per user preference)

**Location:** Lines 252-310 in `main.rs`

### 8. Fix Hardcoded Paths in Icon Generation Scripts

**Files:** `generate-icons-simple.js`, `generate-icons.js`Replace string concatenation with `path.join()` for cross-platform path handling.**Changes:**

- Import `path` module in both scripts
- Replace `'src-tauri/icons/icon.svg'` with `path.join('src-tauri', 'icons', 'icon.svg')`
- Replace template literals like `` `src-tauri/icons/${name}` `` with `path.join('src-tauri', 'icons', name)`

## Implementation Order

1. **Phase 1: Critical Features** (Items 1-5)

- Windows input blocking
- macOS input blocking
- Emergency chord for Windows
- Emergency chord for macOS
- Update command handler

2. **Phase 2: UX Improvements** (Item 6)

- macOS tray behavior

3. **Phase 3: Code Quality** (Items 7-8)

- Linux lock documentation
- Script path fixes

## Testing Requirements

- Test input blocking on Windows (requires admin/elevated permissions for hooks)
- Test input blocking on macOS (requires Accessibility permissions)
- Test emergency chord on all platforms
- Verify macOS tray menu opens on left-click
- Verify other platforms maintain current behavior
- Test icon generation scripts on Windows

## Notes

- Windows input hooks require proper cleanup to avoid resource leaks
- macOS event taps require Accessibility permissions - may need to add permission request UI
- Emergency chord implementation may need to work around input blocking (chord should bypass blocks)