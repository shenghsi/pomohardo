# Implementation Plan

- [ ] 1. Add configuration support for unlock behavior
  - Add `UnlockBehavior` enum to config module with AutoResume and FreshStart variants
  - Add `unlock_behavior` field to Config struct with default implementation
  - Update settings UI to include unlock behavior dropdown
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [ ]* 1.1 Write property test for configuration round-trip
  - **Property 3: Configuration round-trip persistence**
  - **Validates: Requirements 4.2, 4.3, 4.5**

- [ ] 2. Create LockDetector module structure
  - Create new `src-tauri/src/lock_detector.rs` file
  - Define `LockDetector` struct with platform-specific state fields
  - Define `LockEvent` enum (Locked, Unlocked)
  - Implement `new()` constructor
  - Add module to main.rs
  - _Requirements: 10.1_

- [ ] 3. Implement Windows lock detection
  - Implement Windows-specific lock state struct
  - Use WTSRegisterSessionNotification to register for session events
  - Handle WM_WTSSESSION_CHANGE messages for lock/unlock
  - Implement WTSQuerySessionInformation for startup lock state detection
  - Implement callback mechanism for lock events
  - _Requirements: 1.1, 1.4_

- [ ]* 3.1 Write unit test for Windows lock detection on startup
  - Test that app detects locked state when starting while system is locked
  - _Requirements: 1.4_

- [ ] 4. Implement macOS lock detection
  - Implement macOS-specific lock state struct
  - Use NSWorkspace notification center for screen sleep/wake events
  - Subscribe to screensDidSleep and screensDidWake notifications
  - Subscribe to sessionDidBecomeActive for unlock detection
  - Use CGSessionCopyCurrentDictionary for startup lock state detection
  - Implement callback mechanism for lock events
  - _Requirements: 2.1, 2.4_

- [ ]* 4.1 Write unit test for macOS lock detection on startup
  - Test that app detects locked state when starting while system is locked
  - _Requirements: 2.4_

- [ ] 5. Implement Linux lock detection
  - Implement Linux-specific lock state struct using zbus
  - Connect to D-Bus session bus
  - Subscribe to org.freedesktop.ScreenSaver ActiveChanged signal
  - Add fallback for org.gnome.ScreenSaver interface
  - Query GetActive() method for startup lock state detection
  - Implement callback mechanism for lock events
  - _Requirements: 3.1, 3.4_

- [ ]* 5.1 Write unit test for Linux lock detection on startup
  - Test that app detects locked state when starting while system is locked
  - _Requirements: 3.4_

- [ ] 6. Extend Timer Engine with lock-aware state
  - Add `locked`, `lock_timestamp`, and `was_paused_before_lock` fields to TimerEngine
  - Implement `handle_lock()` method
  - Implement `handle_unlock()` method with UnlockAction return type
  - Implement `is_day_boundary_crossed()` helper method
  - _Requirements: 1.2, 1.3, 5.2, 6.2, 8.1_

- [ ]* 6.1 Write property test for work session pause on lock
  - **Property 1: Work session pause on lock**
  - **Validates: Requirements 1.2, 1.3, 2.2, 2.3, 3.2, 3.3**

- [ ]* 6.2 Write property test for lock state persistence
  - **Property 2: Lock state persistence**
  - **Validates: Requirements 1.5, 2.5, 3.5**

- [ ]* 6.3 Write property test for lock duration exclusion
  - **Property 4: Lock duration exclusion**
  - **Validates: Requirements 5.4**

- [ ] 7. Implement work session lock handling
  - In `handle_lock()`, check if current phase is Work
  - If Work phase and Running status, pause the timer
  - Store lock timestamp and previous pause state
  - Set locked flag to true
  - _Requirements: 1.2, 2.2, 3.2_

- [ ] 8. Implement break session lock handling
  - In `handle_lock()`, check if current phase is Break or LongBreak
  - If break phase, do NOT pause the timer (let it continue)
  - Store lock timestamp for day boundary detection
  - Set locked flag to true
  - _Requirements: 7.1_

- [ ]* 8.1 Write property test for break continuation during lock
  - **Property 5: Break continuation during lock**
  - **Validates: Requirements 7.1, 7.4**

- [ ] 9. Implement unlock handling with day boundary detection
  - In `handle_unlock()`, check if day boundary was crossed using lock_timestamp
  - If day boundary crossed, return UnlockAction::StartNewSession
  - Reset session count to zero on day boundary
  - Clear locked flag and lock_timestamp
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ]* 9.1 Write property test for day boundary detection
  - **Property 6: Day boundary detection**
  - **Validates: Requirements 8.1**

- [ ]* 9.2 Write unit test for day boundary unlock behavior
  - Test that unlock after midnight starts fresh session
  - Test that session count resets to zero
  - Test that unlock behavior preference is ignored
  - _Requirements: 8.2, 8.3, 8.4, 8.5_

- [ ] 10. Implement auto-resume unlock behavior
  - In `handle_unlock()`, check unlock_behavior config
  - If AutoResume and not day boundary and was not manually paused, resume timer
  - Return UnlockAction::Resume
  - _Requirements: 5.2, 5.3, 5.4_

- [ ]* 10.1 Write unit test for manual pause preservation
  - Test that manually paused timer stays paused after unlock
  - _Requirements: 5.5_

- [ ] 11. Implement fresh start unlock behavior
  - In `handle_unlock()`, check unlock_behavior config
  - If FreshStart and not day boundary, stop current timer
  - Preserve session count and break debt
  - Return UnlockAction::StartNewSession
  - _Requirements: 6.2, 6.3, 6.4, 6.5_

- [ ]* 11.1 Write property test for session count preservation
  - **Property 9: Session count preservation**
  - **Validates: Requirements 6.5**

- [ ] 12. Implement break completion during lock
  - In timer's check_and_transition(), detect when break completes
  - If system is locked when break completes, pause at zero seconds
  - On unlock, automatically start new work session
  - _Requirements: 7.2, 7.3_

- [ ]* 12.1 Write unit test for break completion during lock
  - Test that break completing while locked pauses at zero
  - Test that unlock after break completion starts work session
  - _Requirements: 7.2, 7.3_

- [ ] 13. Integrate LockDetector with main application
  - Initialize LockDetector in main.rs setup
  - Start monitoring with callback that emits Tauri events
  - Add LockDetector to AppState
  - Handle lock/unlock events in background thread
  - Call timer.handle_lock() on lock events
  - Call timer.handle_unlock() on unlock events
  - _Requirements: 1.1, 2.1, 3.1, 5.1_

- [ ]* 13.1 Write property test for lock event emission
  - **Property 10: Lock event emission**
  - **Validates: Requirements 10.4**

- [ ] 14. Handle BreakShield during system lock
  - Detect lock events in breakshield management code
  - Hide BreakShield window when system locks during break
  - Do not show BreakShield while system is locked
  - _Requirements: 7.5_

- [ ] 15. Preserve continuous running behavior across midnight
  - Verify existing date change logic in timer continues to work
  - Ensure emergency skip counter resets at midnight
  - Ensure daily limit lock clears at midnight
  - Ensure session count continues incrementing without reset
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [ ]* 15.1 Write property test for continuous running across midnight
  - **Property 7: Continuous running across midnight**
  - **Validates: Requirements 9.1, 9.4**

- [ ]* 15.2 Write property test for daily counter reset
  - **Property 8: Daily counter reset on date change**
  - **Validates: Requirements 9.2, 9.3**

- [ ] 16. Add error handling and graceful degradation
  - Wrap lock detection initialization in Result
  - Log errors if lock detection fails to initialize
  - Continue app functionality if lock detection unavailable
  - Add fallback behavior for each platform's API failures
  - _Requirements: All error handling scenarios_

- [ ]* 16.1 Write unit tests for error handling
  - Test graceful degradation when lock detection fails
  - Test fallback behavior for platform API failures
  - Test that errors don't crash the application

- [ ] 17. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
