# Implementation Plan

- [-] 1. Implement Windows input blocking
  - [x] 1.1 Add Windows-specific state struct and imports
    - Add `WindowsHookState` struct with `HHOOK` handles for keyboard and mouse
    - Add `windows` field to `InputBlocker` struct with `#[cfg(target_os = "windows")]`
    - Import required Windows API types from `windows` crate
    - _Requirements: 1.1, 1.2_
  - [-] 1.2 Implement `activate_windows()` with SetWindowsHookEx
    - Install keyboard hook using `SetWindowsHookExW` with `WH_KEYBOARD_LL`
    - Install mouse hook using `SetWindowsHookExW` with `WH_MOUSE_LL`
    - Implement hook callback functions that block events (return non-zero)
    - Store hook handles in `WindowsHookState`
    - Handle partial failure (clean up keyboard hook if mouse hook fails)
    - _Requirements: 1.1, 1.2, 1.4, 1.5_
  - [ ] 1.3 Implement `deactivate_windows()` with UnhookWindowsHookEx
    - Remove keyboard hook using `UnhookWindowsHookEx`
    - Remove mouse hook using `UnhookWindowsHookEx`
    - Clear `WindowsHookState` from struct
    - _Requirements: 1.3_
  - [ ]* 1.4 Write property test for Windows activation round-trip
    - **Property 1: Activation-Deactivation Round Trip**
    - **Validates: Requirements 1.3**

- [ ] 2. Implement macOS input blocking
  - [ ] 2.1 Add macOS-specific state struct and imports
    - Add `MacOSEventTapState` struct with event tap and run loop source references
    - Add `macos` field to `InputBlocker` struct with `#[cfg(target_os = "macos")]`
    - Import required types from `core-graphics` and `core-foundation` crates
    - _Requirements: 2.1_
  - [ ] 2.2 Implement `activate_macos()` with CGEventTapCreate
    - Create event tap using `CGEventTapCreate` with `kCGHeadInsertEventTap`
    - Configure tap to intercept keyboard and mouse events
    - Implement callback that suppresses events (return NULL)
    - Add tap to run loop for event processing
    - Handle permission errors with descriptive message
    - _Requirements: 2.1, 2.2, 2.4, 2.5_
  - [ ] 2.3 Implement `deactivate_macos()` to release event tap
    - Disable event tap using `CGEventTapEnable(tap, false)`
    - Remove from run loop
    - Release event tap reference
    - Clear `MacOSEventTapState` from struct
    - _Requirements: 2.3_
  - [ ]* 2.4 Write property test for macOS activation round-trip
    - **Property 1: Activation-Deactivation Round Trip**
    - **Validates: Requirements 2.3**

- [ ] 3. Checkpoint - Verify input blocking compiles
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 4. Implement Windows emergency chord detection
  - [ ] 4.1 Implement `emergency_chord_pressed()` for Windows
    - Use `GetAsyncKeyState` to query Ctrl, Alt, Shift, and E key states
    - Return true only if all four keys are pressed simultaneously
    - Handle the case where input blocking is active (chord should still be detectable)
    - _Requirements: 3.1, 3.2, 3.3_
  - [ ]* 4.2 Write property test for Windows chord detection
    - **Property 2: Emergency Chord Detection Accuracy**
    - **Validates: Requirements 3.3**

- [ ] 5. Implement macOS emergency chord detection
  - [ ] 5.1 Implement `emergency_chord_pressed()` for macOS
    - Use `CGEventSourceFlagsState` to query modifier key states
    - Query E key state using appropriate Core Graphics API
    - Return true only if Ctrl+Alt+Shift+E are all pressed
    - Handle the case where input blocking is active
    - _Requirements: 4.1, 4.2, 4.3_
  - [ ]* 5.2 Write property test for macOS chord detection
    - **Property 2: Emergency Chord Detection Accuracy**
    - **Validates: Requirements 4.3**

- [ ] 6. Update main.rs emergency chord command
  - [ ] 6.1 Remove platform-specific fallback in emergency_chord_pressed command
    - Remove `#[cfg(not(target_os = "linux"))]` block that returns `Ok(false)`
    - Call `blocker.emergency_chord_pressed()` for all platforms
    - Ensure Windows and macOS implementations are called
    - _Requirements: 3.1, 3.2, 3.3, 4.1, 4.2, 4.3_

- [ ] 7. Checkpoint - Verify emergency chord works
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Implement macOS tray behavior
  - [ ] 8.1 Make tray left-click behavior platform-specific
    - Add `#[cfg(target_os = "macos")]` to set `show_menu_on_left_click(true)`
    - Add `#[cfg(not(target_os = "macos"))]` to set `show_menu_on_left_click(false)`
    - Conditionally disable left-click window toggle handler on macOS
    - _Requirements: 5.1, 5.2, 5.3_

- [ ] 9. Fix icon generation script paths
  - [ ] 9.1 Update generate-icons-simple.js with path.join()
    - Import `path` module
    - Replace `'src-tauri/icons/icon.svg'` with `path.join('src-tauri', 'icons', 'icon.svg')`
    - Replace template literal paths with `path.join()` calls
    - _Requirements: 6.1, 6.3_
  - [ ] 9.2 Update generate-icons.js with path.join()
    - Import `path` module
    - Replace hardcoded paths with `path.join()` calls
    - _Requirements: 6.2, 6.3_
  - [ ]* 9.3 Write property test for path resolution
    - **Property 3: Cross-Platform Path Resolution**
    - **Validates: Requirements 6.1, 6.2, 6.3**

- [ ] 10. Document Linux instance lock
  - [ ] 10.1 Add documentation comments to acquire_instance_lock function
    - Explain why custom lock exists alongside tauri-plugin-single-instance
    - Document the race condition or edge case it addresses
    - Explain the relationship between the two mechanisms
    - _Requirements: 7.1, 7.2_

- [ ] 11. Write idempotency property tests
  - [ ]* 11.1 Write property test for idempotent activation
    - **Property 4: Idempotent Activation**
    - **Validates: Requirements 1.1, 1.2, 2.1**
  - [ ]* 11.2 Write property test for idempotent deactivation
    - **Property 5: Idempotent Deactivation**
    - **Validates: Requirements 1.3, 2.3**

- [ ] 12. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
