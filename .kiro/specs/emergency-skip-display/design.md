# Design Document: Emergency Skip Display

## Overview

This feature adds a visual indicator in the main timer window showing the number of emergency skips remaining for the current day. The display will be positioned below the existing break debt indicator and will update in real-time as emergency skips are used. The backend already tracks emergency skip usage and implements daily reset logic, so this feature primarily involves frontend UI changes to expose this existing data.

## Architecture

The implementation follows the existing application architecture:

- **Backend (Rust)**: The `TimerEngine` in `timer.rs` already tracks `emergency_skips_today`, `emergency_skips_limit`, and implements daily reset logic via `reset_daily_skip_count_if_needed()`. No backend changes are required.
- **Frontend (JavaScript)**: The `main.js` file will be updated to display the emergency skip information from the `TimerState` object that is already being fetched every second.
- **UI (HTML/CSS)**: The `index.html` will add a new display element, and `styles.css` will style it consistently with the existing break debt display.

## Components and Interfaces

### Frontend Components

#### 1. Emergency Skip Display Element (HTML)

A new DOM element will be added to `index.html` in the `.info` section, positioned after the break debt display:

```html
<div class="info">
    <div class="session-count">Session: <span id="sessionCount">0</span></div>
    <div class="break-debt" id="breakDebt">Break debt: 0s</div>
    <div class="emergency-skips" id="emergencySkips">Emergency skips left: 2/2</div>
</div>
```

#### 2. JavaScript Update Logic (main.js)

The `updateUI()` function already receives `TimerState` which includes:
- `emergency_skips_today`: Number of emergency skips used today
- `emergency_skips_limit`: The daily limit (may be locked)

The function will be updated to calculate and display remaining skips:

```javascript
function updateUI(overrideState) {
    const displayState = overrideState || timerState;
    if (!displayState) return;

    // ... existing code ...

    // Emergency skips display
    const skipsUsed = displayState.emergency_skips_today;
    const skipsLimit = displayState.emergency_skips_limit;
    const skipsRemaining = skipsLimit - skipsUsed;
    const emergencySkipsElement = document.getElementById('emergencySkips');
    if (emergencySkipsElement) {
        emergencySkipsElement.textContent = `Emergency skips left: ${skipsRemaining}/${skipsLimit}`;
    }
}
```

#### 3. CSS Styling (styles.css)

The new `.emergency-skips` class will be styled consistently with `.break-debt`:

```css
.emergency-skips {
    font-size: 14px;
    color: #666;
    margin-top: 8px;
}
```

### Backend Components

No backend changes are required. The existing `TimerEngine` already provides:

1. **Daily Reset**: The `reset_daily_skip_count_if_needed()` method checks if the date has changed and resets `emergency_skips_today` to 0
2. **State Exposure**: The `get_state()` method returns `emergency_skips_today` and `emergency_skips_limit` in the `TimerState` struct
3. **Update Frequency**: The frontend calls `get_timer_state` every second via `updateTimerState()`, ensuring the display stays current

## Data Models

### TimerState (Existing)

The `TimerState` struct already includes all necessary fields:

```rust
pub struct TimerState {
    pub phase: Phase,
    pub status: TimerStatus,
    pub remaining_seconds: u32,
    pub total_seconds: u32,
    pub session_count: u32,
    pub break_debt_seconds: u32,
    pub emergency_skips_today: u32,      // Used for display
    pub emergency_skips_limit: u32,      // Used for display
    pub emergency_limit_locked: bool,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Display Format and Accuracy

*For any* timer state with valid emergency skip values, the displayed text should match the format "Emergency skips left: X/Y" where X equals (limit - used) and Y equals the limit, and X should never be negative.

**Validates: Requirements 1.3, 1.4**

### Property 2: State Synchronization

*For any* timer state update, the displayed emergency skip values should exactly match emergency_skips_today and emergency_skips_limit from the backend TimerState, including when the limit is locked.

**Validates: Requirements 3.1, 3.2, 3.3**

### Property 3: Daily Reset Reflection

*For any* timer state after a date change (midnight crossing), the displayed emergency skips used should be 0 and the remaining count should equal the daily limit.

**Validates: Requirements 2.1, 2.2, 2.3**

## Error Handling

### Missing DOM Element

If the `emergencySkips` element is not found (e.g., in the breakshield window which uses a different layout), the code should gracefully skip the update without throwing an error. This is already handled by the null check:

```javascript
if (emergencySkipsElement) {
    emergencySkipsElement.textContent = ...;
}
```

### Invalid State Values

If `emergency_skips_today` or `emergency_skips_limit` are undefined or null, the display should show a safe default (e.g., "Emergency skips left: 0/0") or hide the element. This can be handled with:

```javascript
const skipsUsed = displayState.emergency_skips_today ?? 0;
const skipsLimit = displayState.emergency_skips_limit ?? 0;
```

### Negative Remaining Count

Although the backend should prevent this, if `emergency_skips_today` exceeds `emergency_skips_limit`, the display should show 0 remaining rather than a negative number:

```javascript
const skipsRemaining = Math.max(0, skipsLimit - skipsUsed);
```

## Testing Strategy

### Unit Tests

Unit tests will verify specific examples and edge cases:

1. **Display Format Test**: Verify the text format matches "Emergency skips left: X/Y"
2. **Zero Skips Test**: Verify display shows "0/2" when limit is reached
3. **Full Skips Test**: Verify display shows "2/2" when no skips have been used
4. **Partial Skips Test**: Verify display shows "1/2" when one skip has been used
5. **Negative Prevention Test**: Verify negative remaining counts are clamped to 0

### Property-Based Tests

Property-based tests will verify universal properties across all inputs:

1. **Display Accuracy Property**: For any valid timer state, verify remaining = limit - used
2. **Non-Negative Property**: For any timer state, verify displayed remaining is never negative
3. **Synchronization Property**: For any timer state update, verify displayed values match backend values

### Integration Tests

Integration tests will verify the feature works end-to-end:

1. **Initial Display Test**: Start app and verify emergency skips display appears with correct initial values
2. **Skip Usage Test**: Use an emergency skip and verify the display updates within 1 second
3. **Daily Reset Test**: Simulate date change and verify display resets to full allocation

### Manual Testing

Manual testing will verify visual appearance and user experience:

1. Verify the display is positioned correctly below break debt
2. Verify the styling matches the existing UI aesthetic
3. Verify the display is readable and clear
4. Verify the display updates smoothly without flickering
