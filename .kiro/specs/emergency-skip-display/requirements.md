# Requirements Document

## Introduction

This feature adds a visual display of remaining emergency skips in the main timer window, positioned below the break debt indicator. Emergency skips are a limited resource that users can use to skip break periods, and displaying the remaining count helps users make informed decisions about when to use them.

## Glossary

- **Emergency_Skip**: A mechanism allowing users to skip mandatory break periods by using one of their daily allotted skips
- **Main_Window**: The primary application window displaying the timer, controls, and status information
- **Break_Debt**: The accumulated time from skipped or shortened breaks that will be added to future break durations
- **Daily_Reset**: The automatic reset of the emergency skip counter that occurs at midnight local time
- **Timer_State**: The current state of the application including phase, remaining time, session count, break debt, and emergency skip information

## Requirements

### Requirement 1: Display Emergency Skips Remaining

**User Story:** As a user, I want to see how many emergency skips I have left today, so that I can make informed decisions about when to use them.

#### Acceptance Criteria

1. WHEN the main window is displayed, THE Main_Window SHALL show the number of emergency skips remaining for the current day
2. THE Main_Window SHALL display the emergency skips remaining below the break debt indicator
3. THE Main_Window SHALL format the display as "Emergency skips left: X/Y" where X is remaining skips and Y is the daily limit
4. WHEN an emergency skip is used, THE Main_Window SHALL immediately update the displayed count to reflect the new remaining amount
5. WHEN the daily limit is reached (0 skips remaining), THE Main_Window SHALL still display "Emergency skips left: 0/Y" to inform the user

### Requirement 2: Daily Reset of Emergency Skip Count

**User Story:** As a user, I want my emergency skip count to reset every day at midnight, so that I have a fresh allocation each day.

#### Acceptance Criteria

1. WHEN the local time crosses midnight (00:00), THE Timer_State SHALL reset the emergency skips used count to 0
2. WHEN the emergency skip count is reset, THE Main_Window SHALL update the display to show the full daily allocation
3. WHEN the application is running during the midnight transition, THE Timer_State SHALL detect the date change and perform the reset automatically
4. WHEN the application is closed and reopened on a new day, THE Timer_State SHALL initialize with the emergency skip count reset to 0

### Requirement 3: Synchronization with Backend State

**User Story:** As a user, I want the emergency skip display to always show accurate information, so that I can trust the displayed count.

#### Acceptance Criteria

1. WHEN the timer state is updated, THE Main_Window SHALL retrieve the current emergency skip information from the backend
2. THE Main_Window SHALL display emergency_skips_today and emergency_skips_limit from the Timer_State
3. WHEN the emergency skip limit is locked for the day, THE Main_Window SHALL display the locked limit value
4. THE Main_Window SHALL update the emergency skip display at least once per second along with other timer information
