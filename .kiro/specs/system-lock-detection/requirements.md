# Requirements Document

## Introduction

This specification addresses the need for Pomohardo to respond intelligently to system lock/unlock events. Currently, when a user locks their system or the system auto-locks, the timer continues running, which results in inaccurate time tracking since no actual work is being done. This feature will detect system lock events, automatically pause the timer, and provide configurable behavior for resuming when the system unlocks.

## Glossary

- **Pomohardo**: The cross-platform pomodoro timer application built with Tauri
- **System Lock**: When the user locks their computer screen (via lock screen, sleep, or screensaver with password)
- **System Unlock**: When the user unlocks their computer by entering credentials
- **Timer Engine**: The core component managing work/break sessions and time tracking
- **Auto-Resume**: Automatically continuing the timer from where it paused when the system unlocks
- **Fresh Start**: Starting a new work session instead of resuming the previous one

## Requirements

### Requirement 1

**User Story:** As a Windows user, I want the timer to pause when I lock my system, so that locked time is not counted as work time.

#### Acceptance Criteria

1. WHEN the system locks on Windows THEN the Pomohardo application SHALL detect the lock event
2. WHEN a lock event is detected on Windows THEN the Timer Engine SHALL pause the active timer if it is running
3. WHEN the timer is paused due to system lock THEN the Timer Engine SHALL preserve the current phase and remaining time
4. WHEN the system is already locked and Pomohardo starts THEN the application SHALL detect the locked state
5. WHILE the system is locked THEN the Timer Engine SHALL remain in paused state

### Requirement 2

**User Story:** As a macOS user, I want the timer to pause when I lock my system, so that locked time is not counted as work time.

#### Acceptance Criteria

1. WHEN the system locks on macOS THEN the Pomohardo application SHALL detect the lock event
2. WHEN a lock event is detected on macOS THEN the Timer Engine SHALL pause the active timer if it is running
3. WHEN the timer is paused due to system lock THEN the Timer Engine SHALL preserve the current phase and remaining time
4. WHEN the system is already locked and Pomohardo starts THEN the application SHALL detect the locked state
5. WHILE the system is locked THEN the Timer Engine SHALL remain in paused state

### Requirement 3

**User Story:** As a Linux user, I want the timer to pause when I lock my system, so that locked time is not counted as work time.

#### Acceptance Criteria

1. WHEN the system locks on Linux THEN the Pomohardo application SHALL detect the lock event via D-Bus session signals
2. WHEN a lock event is detected on Linux THEN the Timer Engine SHALL pause the active timer if it is running
3. WHEN the timer is paused due to system lock THEN the Timer Engine SHALL preserve the current phase and remaining time
4. WHEN the system is already locked and Pomohardo starts THEN the application SHALL detect the locked state via D-Bus
5. WHILE the system is locked THEN the Timer Engine SHALL remain in paused state

### Requirement 4

**User Story:** As a user, I want to configure whether the timer auto-resumes or starts fresh when I unlock my system, so that I can choose the behavior that fits my workflow.

#### Acceptance Criteria

1. WHEN the user opens settings THEN the application SHALL display a configuration option for unlock behavior
2. WHEN the user selects "Auto-resume" mode THEN the configuration SHALL store this preference
3. WHEN the user selects "Fresh start" mode THEN the configuration SHALL store this preference
4. WHEN no preference is set THEN the application SHALL default to "Auto-resume" mode
5. WHEN the configuration is saved THEN the unlock behavior preference SHALL persist across application restarts

### Requirement 5

**User Story:** As a user with auto-resume enabled, I want the timer to continue from where it paused when I unlock my system, so that I can seamlessly continue my work session.

#### Acceptance Criteria

1. WHEN the system unlocks on any platform THEN the Pomohardo application SHALL detect the unlock event
2. WHEN unlock is detected and auto-resume is enabled and the timer was paused due to lock THEN the Timer Engine SHALL resume the timer
3. WHEN the timer resumes after unlock THEN the Timer Engine SHALL continue from the same phase and remaining time as when it was paused
4. WHEN the timer resumes after unlock THEN the paused duration SHALL not count toward the session time
5. IF the timer was manually paused before system lock THEN the timer SHALL remain paused after unlock

### Requirement 6

**User Story:** As a user with fresh start enabled, I want a new work session to begin when I unlock my system, so that each unlock represents a fresh focus period.

#### Acceptance Criteria

1. WHEN the system unlocks on any platform THEN the Pomohardo application SHALL detect the unlock event
2. WHEN unlock is detected and fresh start is enabled and the timer was paused due to lock THEN the Timer Engine SHALL stop the current timer
3. WHEN fresh start mode activates THEN the Timer Engine SHALL reset to a new work session
4. WHEN fresh start mode activates THEN the Timer Engine SHALL set the timer to stopped state
5. WHEN fresh start mode activates THEN the session count and break debt SHALL be preserved

### Requirement 7

**User Story:** As a user, I want break sessions to continue during system lock, so that I am encouraged to take full breaks away from my computer.

#### Acceptance Criteria

1. WHEN the system locks during a break session THEN the Timer Engine SHALL continue running the break timer
2. WHEN break time completes while the system is locked THEN the Timer Engine SHALL transition to paused state at zero seconds
3. WHEN the system unlocks after break completion THEN the Timer Engine SHALL automatically start a new work session
4. WHEN the system unlocks during an active break THEN the Timer Engine SHALL continue the break timer and the breakshield should be displayed
5. WHILE the system is locked during a break THEN the BreakShield overlay SHALL not be displayed

### Requirement 8

**User Story:** As a user who locks my system overnight, I want a fresh work session to start when I unlock the next day, so that I don't resume yesterday's stale session.

#### Acceptance Criteria

1. WHEN the system unlocks and the current date differs from the lock date THEN the Timer Engine SHALL detect a day boundary crossing
2. WHEN a day boundary is detected during unlock THEN the Timer Engine SHALL discard the paused session state
3. WHEN a day boundary is detected during unlock THEN the Timer Engine SHALL start a new work session automatically
4. WHEN a day boundary is detected during unlock THEN the Timer Engine SHALL reset session count to zero
5. WHEN a day boundary is detected during unlock THEN the unlock behavior preference SHALL be ignored

### Requirement 9

**User Story:** As a user who leaves a work session running overnight without locking, I want the current behavior to be maintained, so that the timer continues tracking time across day boundaries.

#### Acceptance Criteria

1. WHEN a work session is running and the date changes THEN the Timer Engine SHALL continue running the session
2. WHEN a work session is running and the date changes THEN the emergency skip counter SHALL reset to zero for the new day
3. WHEN a work session is running and the date changes THEN the daily limit lock SHALL be cleared for the new day
4. WHEN a work session completes after a date change THEN the Timer Engine SHALL transition to break normally
5. WHEN the application is running continuously across midnight THEN the session count SHALL continue incrementing normally without reset

### Requirement 10

**User Story:** As a developer, I want the system lock detection to be testable, so that I can verify correct behavior across platforms.

#### Acceptance Criteria

1. WHEN the lock detection module is initialized THEN the module SHALL expose methods for querying lock state
2. WHEN testing lock detection THEN the module SHALL provide a way to simulate lock events
3. WHEN testing unlock detection THEN the module SHALL provide a way to simulate unlock events
4. WHEN the lock state changes THEN the module SHALL emit events that can be observed by tests
5. WHEN running in test mode THEN the module SHALL not require actual system lock/unlock
