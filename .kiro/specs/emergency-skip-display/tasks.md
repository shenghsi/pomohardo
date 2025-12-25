# Implementation Plan: Emergency Skip Display

## Overview

This implementation adds a visual display of remaining emergency skips in the main timer window. The backend already tracks all necessary data, so this is primarily a frontend UI enhancement. The implementation will add a new DOM element, update the JavaScript to populate it with data from the existing TimerState, and style it consistently with the existing UI.

## Tasks

- [x] 1. Add emergency skip display element to HTML
  - Add a new `<div>` element with id "emergencySkips" in the `.info` section of `index.html`
  - Position it after the break debt display element
  - Set initial text content to "Emergency skips left: 2/2" as a placeholder
  - _Requirements: 1.1, 1.2_

- [x] 2. Update JavaScript to populate emergency skip display
  - [x] 2.1 Add DOM element reference in `initDOMElements()` function
    - Add `emergencySkips` variable declaration at the top of `main.js`
    - Add `emergencySkips = document.getElementById('emergencySkips');` in `initDOMElements()`
    - _Requirements: 1.1_

  - [x] 2.2 Update `updateUI()` function to display emergency skip data
    - Calculate remaining skips: `skipsRemaining = Math.max(0, skipsLimit - skipsUsed)`
    - Format display text: `Emergency skips left: ${skipsRemaining}/${skipsLimit}`
    - Update the DOM element with null check: `if (emergencySkipsElement) { ... }`
    - Use nullish coalescing for safe defaults: `displayState.emergency_skips_today ?? 0`
    - _Requirements: 1.3, 1.4, 1.5, 3.2_

  - [ ]* 2.3 Write property test for display format and accuracy
    - **Property 1: Display Format and Accuracy**
    - **Validates: Requirements 1.3, 1.4**
    - Generate random timer states with various emergency skip values
    - Verify displayed text matches format "Emergency skips left: X/Y"
    - Verify X = max(0, limit - used) and Y = limit
    - Test edge cases: used=0, used=limit, used>limit

  - [ ]* 2.4 Write property test for state synchronization
    - **Property 2: State Synchronization**
    - **Validates: Requirements 3.1, 3.2, 3.3**
    - Generate random timer states with emergency skip data
    - Call updateUI() with each state
    - Verify displayed values match emergency_skips_today and emergency_skips_limit
    - Test with emergency_limit_locked both true and false

- [x] 3. Add CSS styling for emergency skip display
  - Add `.emergency-skips` class to `styles.css`
  - Set font-size to 14px to match break debt display
  - Set color to #666 for consistency
  - Add margin-top of 8px for spacing
  - _Requirements: 1.2_

- [ ]* 4. Write integration tests
  - Test initial display on app load shows correct values
  - Test display updates when emergency skip is used
  - Test display shows 0 remaining when limit is reached
  - _Requirements: 1.1, 1.4, 1.5_

- [ ]* 5. Write property test for daily reset reflection
  - **Property 3: Daily Reset Reflection**
  - **Validates: Requirements 2.1, 2.2, 2.3**
  - Simulate timer states before and after midnight
  - Verify that after date change, displayed used count is 0
  - Verify remaining count equals the daily limit after reset

- [ ] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- The backend already implements all necessary logic for tracking and resetting emergency skips
- The `updateUI()` function is called every second, so the display will automatically stay synchronized
- The display will only appear in the main window, not in the breakshield window (which has a different layout)
- Property tests should use a JavaScript property-based testing library like fast-check
