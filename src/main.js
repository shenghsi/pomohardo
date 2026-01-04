// Wait for Tauri API to be available
let invoke;
const IS_BREAKSHIELD = window.location.hash === '#breakshield';
let breakshieldOpen = false; // main-window guard: don't spam open/close each second

// State
let timerState = null;
let config = null;
let updateInterval = null;
let frozenWorkState = null; // Stores last work state to freeze main window during breaks

// DOM Elements (will be initialized after DOM is ready)
let tabs, tabContents, timeDisplay, phaseLabel, sessionCount, breakDebt, emergencySkips;
let progressCircle, pauseBtn, pauseIcon, skipBtn, settingsBtn, breakOverlay;
let breakTimeDisplay, breakPhaseLabel, breakTimeUpContainer, breakTimer;
let emergencySkipContainer, holdProgressBar, holdProgressText;
let confirmWordContainer, confirmWordInput, confirmSkipBtn, confirmInstruction;
let emergencyLimitMsg;
let emergencyHoldInstruction;
let holdProgressContainer;

// Get platform-specific emergency key combo text
function getEmergencyKeyCombo() {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('mac')) {
        return 'Cmd+Option+Shift+E';
    }
    return 'Ctrl+Alt+Shift+E';
}

// Initialize DOM elements
function initDOMElements() {
    tabs = document.querySelectorAll('.tab');
    tabContents = document.querySelectorAll('.tab-content');
    timeDisplay = document.getElementById('timeDisplay');
    phaseLabel = document.getElementById('phaseLabel');
    sessionCount = document.getElementById('sessionCount');
    breakDebt = document.getElementById('breakDebt');
    emergencySkips = document.getElementById('emergencySkips');
    progressCircle = document.getElementById('progressCircle');
    pauseBtn = document.getElementById('pauseBtn');
    pauseIcon = document.getElementById('pauseIcon');
    skipBtn = document.getElementById('skipBtn');
    settingsBtn = document.getElementById('settingsBtn');
}

// Calculate breakshield sizes based on screen dimensions
function calculateBreakshieldSizes() {
    const width = window.innerWidth;
    const height = window.innerHeight;
    
    // Base sizes as percentages of screen dimensions
    // Break timer container: 30-40% of screen width, with min/max constraints
    const breakTimerWidth = width * 0.2;
    
    // Break content padding: responsive to screen size
    const breakContentPaddingX = width * 0.04;
    const breakContentPaddingY = height * 0.02;
    
    // Time display font size: scales with screen height
    const timeDisplayFontSize = height * 0.05;
    const timeDisplayMinWidth = width * 0.2;
    
    // Phase label font size: scales with screen height
    const phaseLabelFontSize = height * 0.015;
    
    // Break message font size: scales with screen height
    const breakMessageFontSize = height * 0.015;
    
    // Break time up container: 35-45% of screen width
    const breakTimeUpWidth = width * 0.25;
    const breakTimeUpFontSize = height * 0.018;
    
    // Emergency skip container: same width as break timer
    const emergencySkipWidth = breakTimerWidth;
    
    // Hold progress: same width as break timer
    const holdProgressWidth = breakTimerWidth;
    
    // Confirm word container: same width as break timer
    const confirmWordWidth = breakTimerWidth;
    const confirmWordInputWidth = breakTimerWidth * 0.6;
    
    // Set CSS custom properties
    const root = document.documentElement;
    root.style.setProperty('--break-timer-width', `${breakTimerWidth}px`);
    root.style.setProperty('--break-content-padding-x', `${breakContentPaddingX}px`);
    root.style.setProperty('--break-content-padding-y', `${breakContentPaddingY}px`);
    root.style.setProperty('--break-time-display-font-size', `${timeDisplayFontSize}px`);
    root.style.setProperty('--break-time-display-min-width', `${timeDisplayMinWidth}px`);
    root.style.setProperty('--break-phase-label-font-size', `${phaseLabelFontSize}px`);
    root.style.setProperty('--break-message-font-size', `${breakMessageFontSize}px`);
    root.style.setProperty('--break-time-up-width', `${breakTimeUpWidth}px`);
    root.style.setProperty('--break-time-up-font-size', `${breakTimeUpFontSize}px`);
    root.style.setProperty('--emergency-skip-width', `${emergencySkipWidth}px`);
    root.style.setProperty('--hold-progress-width', `${holdProgressWidth}px`);
    root.style.setProperty('--confirm-word-width', `${confirmWordWidth}px`);
    root.style.setProperty('--confirm-word-input-width', `${confirmWordInputWidth}px`);
}

// Create break screen DOM (only called in breakshield mode)
function createBreakScreen() {
    const html = `
        <div class="break-overlay hidden" id="breakOverlay">
            <div class="break-timer">
                <div class="break-content">
                    <div class="phase-label" id="breakPhaseLabel">Break</div>
                    <div class="time-display" id="breakTimeDisplay">5:00</div>
                </div>
                <p class="break-message">Take a break!<br>Step away from your computer.</p>
                <p class="emergency-limit-msg hidden" id="emergencyLimitMsg">
                    Emergency skip limit reached for today.
                </p>
            </div>
            <div class="emergency-skip-container hidden" id="emergencySkipContainer">
                <p class="emergency-instruction" id="emergencyHoldInstruction">
                    Hold <kbd id="emergencyKeyCombo">Ctrl+Alt+Shift+E</kbd> for 4 seconds to arm emergency skip
                </p>
                <div class="hold-progress" id="holdProgressContainer">
                    <div class="hold-progress-bar" id="holdProgressBar"></div>
                    <span id="holdProgressText">0%</span>
                </div>
            </div>
            <div class="confirm-word-container hidden" id="confirmWordContainer">
                <p class="confirm-instruction" id="confirmInstruction">Type <strong>SKIPBREAK</strong> to confirm:</p>
                <input type="text" id="confirmWordInput" class="confirm-word-input" autocomplete="off" />
                <button class="confirm-btn" id="confirmSkipBtn">Confirm</button>
            </div>

            <div class="break-time-up-container hidden" id="breakTimeUpContainer">
                <p class="break-time-up-msg">Break complete!<br>Left click or press any key to start work session.</p>
            </div>
        </div>
    `;
    document.body.insertAdjacentHTML('beforeend', html);

    // Now get references to the created elements
    breakOverlay = document.getElementById('breakOverlay');
    breakTimer = document.querySelector('.break-timer');
    breakTimeDisplay = document.getElementById('breakTimeDisplay');
    breakPhaseLabel = document.getElementById('breakPhaseLabel');
    breakTimeUpContainer = document.getElementById('breakTimeUpContainer');
    emergencyLimitMsg = document.getElementById('emergencyLimitMsg');
    emergencySkipContainer = document.getElementById('emergencySkipContainer');
    emergencyHoldInstruction = document.getElementById('emergencyHoldInstruction');
    holdProgressContainer = document.getElementById('holdProgressContainer');
    holdProgressBar = document.getElementById('holdProgressBar');
    holdProgressText = document.getElementById('holdProgressText');
    confirmWordContainer = document.getElementById('confirmWordContainer');
    confirmWordInput = document.getElementById('confirmWordInput');
    confirmSkipBtn = document.getElementById('confirmSkipBtn');
    confirmInstruction = document.getElementById('confirmInstruction');
    
    // Update key combo text for platform
    const emergencyKeyCombo = document.getElementById('emergencyKeyCombo');
    if (emergencyKeyCombo) {
        emergencyKeyCombo.textContent = getEmergencyKeyCombo();
    }
    
    // Calculate and set sizes based on screen dimensions
    calculateBreakshieldSizes();
    
    // Recalculate on window resize
    window.addEventListener('resize', calculateBreakshieldSizes);
}

// Tab switching
function setupEventListeners() {
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;
            tabs.forEach(t => t.classList.remove('active'));
            tabContents.forEach(c => c.classList.remove('active'));
            tab.classList.add('active');
            document.getElementById(tabName + 'Tab').classList.add('active');

            if (tabName === 'stats') {
                updateStats();
            }
        });
    });

    // Settings window (opens as popup)
    settingsBtn.addEventListener('click', () => {
        openSettings();
    });

    // Control buttons
    pauseBtn.addEventListener('click', async () => {
        if (!timerState) return;

        if (timerState.status === 'Running') {
            await invoke('pause_timer');
            pauseIcon.src = 'play.svg';
        } else if (timerState.status === 'Paused') {
            await invoke('resume_timer');
            pauseIcon.src = 'pause.svg';
        } else {
            await invoke('start_timer');
            pauseIcon.src = 'pause.svg';
        }

        await updateTimerState();
    });

    skipBtn.addEventListener('click', async () => {
        if (!timerState || timerState.phase !== 'Work') return;

        try {
            await invoke('skip_work');
            await updateTimerState();
        } catch (error) {
            console.error('Skip failed:', error);
        }
    });

    // Emergency skip with hold chord + confirm word (breakshield only)
    if (IS_BREAKSHIELD) {
        setupEmergencySkip();
    }
}

// Setup emergency skip with hold chord detection
function setupEmergencySkip() {
    let holdStartTime = null;
    let holdInterval = null;
    let isHolding = false;
    let isArmed = false;  // true once hold is complete and confirm word input is shown
    let confirmTimeout = null;  // timeout for confirm word input
    
    // JavaScript-side key detection for macOS (and fallback for other platforms)
    let keyState = {
        cmd: false,
        ctrl: false,
        alt: false,
        option: false,
        shift: false,
        e: false
    };
    
    // Global key event listeners to track key state
    const keyDownHandler = (e) => {
        // Prevent default behavior for our emergency combo
        const platform = navigator.platform.toLowerCase();
        const isEmergencyCombo = platform.includes('mac') 
            ? (e.metaKey && e.altKey && e.shiftKey && e.key.toLowerCase() === 'e')
            : (e.ctrlKey && e.altKey && e.shiftKey && e.key.toLowerCase() === 'e');
            
        if (isEmergencyCombo) {
            e.preventDefault();
            e.stopPropagation();
        }
        
        keyState.cmd = e.metaKey;
        keyState.ctrl = e.ctrlKey;
        keyState.alt = e.altKey;
        keyState.option = e.altKey; // Alt and Option are the same key
        keyState.shift = e.shiftKey;
        if (e.key.toLowerCase() === 'e') {
            keyState.e = true;
        }
        
        // Debug logging for macOS
        if (platform.includes('mac') && (e.metaKey || e.altKey || e.shiftKey || e.key.toLowerCase() === 'e')) {
            console.log('Key state:', {
                cmd: keyState.cmd,
                option: keyState.option,
                shift: keyState.shift,
                e: keyState.e,
                key: e.key,
                combo: isEmergencyCombo
            });
        }
    };
    
    const keyUpHandler = (e) => {
        keyState.cmd = e.metaKey;
        keyState.ctrl = e.ctrlKey;
        keyState.alt = e.altKey;
        keyState.option = e.altKey;
        keyState.shift = e.shiftKey;
        if (e.key.toLowerCase() === 'e') {
            keyState.e = false;
        }
    };
    
    // Add global key listeners with capture to catch events before they're blocked
    document.addEventListener('keydown', keyDownHandler, { capture: true, passive: false });
    document.addEventListener('keyup', keyUpHandler, { capture: true, passive: false });
    
    // Also add to window for broader coverage
    window.addEventListener('keydown', keyDownHandler, { capture: true, passive: false });
    window.addEventListener('keyup', keyUpHandler, { capture: true, passive: false });
    
    // Function to check if the correct key combination is pressed
    const isEmergencyChordPressed = () => {
        const platform = navigator.platform.toLowerCase();
        if (platform.includes('mac')) {
            // macOS: Cmd+Option+Shift+E
            const result = keyState.cmd && keyState.option && keyState.shift && keyState.e;
            if (result) {
                console.log('Emergency chord detected on macOS!');
            }
            return result;
        } else {
            // Windows/Linux: Ctrl+Alt+Shift+E
            return keyState.ctrl && keyState.alt && keyState.shift && keyState.e;
        }
    };

    // Show emergency skip UI when break overlay is shown
    const showEmergencySkipUI = () => {
        // If we're already at the confirm step, don't reset the UI.
        if (confirmWordContainer && !confirmWordContainer.classList.contains('hidden')) {
            return;
        }
        if (emergencyLimitMsg) emergencyLimitMsg.classList.add('hidden');
        if (emergencyHoldInstruction) emergencyHoldInstruction.classList.remove('hidden');
        if (holdProgressContainer) holdProgressContainer.classList.remove('hidden');
        emergencySkipContainer.classList.remove('hidden');
        confirmWordContainer.classList.add('hidden');
        holdProgressBar.style.width = '0%';
        holdProgressText.textContent = '0%';
    };

    // Hide emergency skip UI when break overlay is hidden
    const hideEmergencySkipUI = () => {
        emergencySkipContainer.classList.add('hidden');
        confirmWordContainer.classList.add('hidden');
        if (holdInterval) {
            clearInterval(holdInterval);
            holdInterval = null;
        }
        if (confirmTimeout) {
            clearTimeout(confirmTimeout);
            confirmTimeout = null;
        }
        isHolding = false;
        isArmed = false;
    };

    // Reset to hold state (called when confirm timeout expires)
    const resetToHoldState = () => {
        if (confirmTimeout) {
            clearTimeout(confirmTimeout);
            confirmTimeout = null;
        }
        isArmed = false;
        confirmWordContainer.classList.add('hidden');
        confirmWordInput.value = '';
        if (emergencyHoldInstruction) emergencyHoldInstruction.classList.remove('hidden');
        if (holdProgressContainer) holdProgressContainer.classList.remove('hidden');
        holdProgressBar.style.width = '0%';
        holdProgressText.textContent = '0%';
    };

    const showEmergencyLimitMsg = () => {
        hideEmergencySkipUI();
        if (emergencyLimitMsg) emergencyLimitMsg.classList.remove('hidden');
    };

    // Check if emergency skip UI should be shown
    const checkAndShowEmergencySkip = async () => {
        try {
            // If confirm box is already shown, do not touch the emergency UI.
            // updateTimerState runs every second; resetting here would make the input disappear.
            if (confirmWordContainer && !confirmWordContainer.classList.contains('hidden')) {
                return;
            }

            const currentState = await invoke('get_timer_state');
            const currentConfig = await invoke('get_config');

            // Only show if limit not reached
            if (currentState.emergency_skips_today >= currentConfig.emergency_skips_per_day) {
                showEmergencyLimitMsg();
            } else {
                showEmergencySkipUI();
            }
        } catch (error) {
            console.error('Failed to check emergency skip:', error);
        }
    };

    // When X11 grabs are active, the webview may not receive key events.
    // So we poll the backend for the chord state (Ctrl+Alt+Shift+E) and implement hold logic here.
    let chordPoll = null;
    let lastPressedAt = null;

    const startChordPolling = async () => {
        if (chordPoll) return;
        chordPoll = setInterval(async () => {
            try {
                if (!breakOverlay || breakOverlay.classList.contains('hidden')) return;
                if (!emergencySkipContainer || emergencySkipContainer.classList.contains('hidden')) return;
                if (isArmed) return;

                // Try backend detection first (works for all platforms now)
                let pressed = false;
                try {
                    pressed = await invoke('emergency_chord_pressed');
                } catch (e) {
                    console.error('Backend chord detection failed:', e);
                    // Fall back to JavaScript detection
                    pressed = isEmergencyChordPressed();
                }
                
                // If backend returns false, also try JavaScript as fallback
                if (!pressed) {
                    pressed = isEmergencyChordPressed();
                }
                
                // Debug logging
                if (pressed) {
                    console.log('Emergency chord detected!');
                }

                const currentConfig = await invoke('get_config');
                const holdDuration = (currentConfig.emergency_hold_seconds || 4) * 1000;

                if (pressed) {
                    if (!lastPressedAt) lastPressedAt = Date.now();
                    const elapsed = Date.now() - lastPressedAt;
                    const progress = Math.min((elapsed / holdDuration) * 100, 100);
                    holdProgressBar.style.width = progress + '%';
                    holdProgressText.textContent = Math.round(progress) + '%';

                    if (progress >= 100) {
                        isArmed = true;
                        // Stop polling while confirm is active
                        clearInterval(chordPoll);
                        chordPoll = null;

                        if (emergencyHoldInstruction) emergencyHoldInstruction.classList.add('hidden');
                        if (holdProgressContainer) holdProgressContainer.classList.add('hidden');
                        confirmWordContainer.classList.remove('hidden');

                        // Update the confirm instruction with the configured word
                        if (confirmInstruction) {
                            confirmInstruction.innerHTML = `Type <strong>${currentConfig.emergency_confirm_word || 'SKIPBREAK'}</strong> to confirm:`;
                        }

                        // Temporarily allow typing by releasing grabs
                        try { await invoke('deactivate_input_blocking'); } catch (_) { }

                        confirmWordInput.value = '';
                        setTimeout(() => confirmWordInput.focus(), 100);

                        // Start confirm timeout
                        const timeoutSeconds = currentConfig.emergency_confirm_timeout_seconds || 15;
                        confirmTimeout = setTimeout(async () => {
                            resetToHoldState();
                            // Re-enable blocking if still in break
                            try { await invoke('activate_input_blocking'); } catch (_) { }
                            startChordPolling();
                        }, timeoutSeconds * 1000);
                    }
                } else {
                    lastPressedAt = null;
                    holdProgressBar.style.width = '0%';
                    holdProgressText.textContent = '0%';
                }
            } catch (e) {
                // If the backend can't poll, just ignore; the UI stays in "hold" state.
            }
        }, 100);
    };

    const stopChordPolling = () => {
        if (chordPoll) {
            clearInterval(chordPoll);
            chordPoll = null;
        }
        lastPressedAt = null;
    };

    // Prevent global keyboard handlers from interfering with the input
    confirmWordInput.addEventListener('keydown', (e) => {
        e.stopPropagation();
    });

    confirmWordInput.addEventListener('keyup', (e) => {
        e.stopPropagation();
    });

    // Confirm word input handler
    confirmSkipBtn.addEventListener('click', async () => {
        await handleEmergencySkipConfirm();
    });

    confirmWordInput.addEventListener('keypress', async (e) => {
        e.stopPropagation();
        if (e.key === 'Enter') {
            await handleEmergencySkipConfirm();
        }
    });

    // Handle emergency skip confirmation
    const handleEmergencySkipConfirm = async () => {
        try {
            const currentConfig = await invoke('get_config');
            const inputWord = confirmWordInput.value.trim().toUpperCase();
            const expectedWord = currentConfig.emergency_confirm_word.toUpperCase();

            if (inputWord !== expectedWord) {
                alert(`Incorrect confirmation word. Expected: ${expectedWord}`);
                confirmWordInput.value = '';
                confirmWordInput.focus();
                return;
            }

            // Check limit again before proceeding
            const currentState = await invoke('get_timer_state');
            if (currentState.emergency_skips_today >= currentConfig.emergency_skips_per_day) {
                showEmergencyLimitMsg();
                return;
            }

            const approved = await invoke('request_emergency_skip');
            if (!approved) {
                showEmergencyLimitMsg();
            } else {
                await updateTimerState();
            }

            // Clear timeout if confirmation succeeds
            if (confirmTimeout) {
                clearTimeout(confirmTimeout);
                confirmTimeout = null;
            }

            // Re-enable blocking if we are still in break and skip was denied.
            // If skip succeeded, updateTimerState() will hide overlay and deactivate blocking anyway.
            if (!approved) {
                try { await invoke('activate_input_blocking'); } catch (_) { }
                startChordPolling();
            }

            // Reset UI
            confirmWordInput.value = '';
            isArmed = false;
            hideEmergencySkipUI();
        } catch (error) {
            console.error('Emergency skip failed:', error);
        }
    };

    // Expose functions to be called from showBreakOverlay/hideBreakOverlay.
    // IMPORTANT: updateTimerState/showBreakOverlay calls this every second; keep it idempotent.
    window.checkAndShowEmergencySkip = async () => {
        await checkAndShowEmergencySkip();
        // If emergency UI is visible, start polling.
        if (emergencySkipContainer && !emergencySkipContainer.classList.contains('hidden')) {
            startChordPolling();
        } else {
            stopChordPolling();
        }
    };

    window.hideEmergencySkipUI = () => {
        stopChordPolling();
        hideEmergencySkipUI();
    };
    
    // Cleanup function to remove event listeners
    window.cleanupEmergencySkip = () => {
        document.removeEventListener('keydown', keyDownHandler, { capture: true });
        document.removeEventListener('keyup', keyUpHandler, { capture: true });
        window.removeEventListener('keydown', keyDownHandler, { capture: true });
        window.removeEventListener('keyup', keyUpHandler, { capture: true });
        stopChordPolling();
        hideEmergencySkipUI();
    };
}

// Update timer state
async function updateTimerState() {
    try {
        timerState = await invoke('get_timer_state');

        // Check if we're in a break phase (regardless of running/paused status)
        const inBreakPhase = (timerState.phase === 'Break' || timerState.phase === 'LongBreak');
        const inBreak = inBreakPhase && timerState.status === 'Running';
        const breakTimeUp = inBreakPhase && timerState.status === 'Paused' && timerState.remaining_seconds === 0;

        // Main window: freeze display during breaks, only manage breakshield window
        if (!IS_BREAKSHIELD) {
            // Capture work state when entering break
            if (inBreakPhase && !frozenWorkState) {
                // Store the state we want to display while frozen
                // Use the configured work duration as the "full" display
                frozenWorkState = {
                    phase: 'Work',
                    status: 'Paused', // Show as paused so user knows it's frozen
                    remaining_seconds: 0, // Show 0:00 to indicate work session ended
                    total_seconds: timerState.total_seconds || 1500,
                    session_count: timerState.session_count,
                    break_debt_seconds: timerState.break_debt_seconds
                };
            }

            // Clear frozen state when work resumes
            if (!inBreakPhase && frozenWorkState) {
                frozenWorkState = null;
            }

            // Update UI with frozen state during breaks, real state otherwise
            updateUI(inBreakPhase ? frozenWorkState : null);

            // Manage breakshield window open/close
            // Keep breakshield open for the entire break phase (even when paused)
            if ((inBreak || breakTimeUp) && !breakshieldOpen) {
                const base = window.location.href.replace(/#.*$/, '');
                await invoke('show_breakshield', { url: `${base}#breakshield` });
                breakshieldOpen = true;
            } else if (!inBreakPhase && breakshieldOpen) {
                await invoke('hide_breakshield');
                breakshieldOpen = false;
            }
            return;
        }

        // Breakshield window: update UI and show/hide break overlay
        updateUI();

        if (timerState.phase === 'Break' || timerState.phase === 'LongBreak') {
            if (timerState.status === 'Running' || breakTimeUp) {
                if (breakOverlay.classList.contains('hidden')) {
                    await showBreakOverlay();
                } else if (breakTimeUp) {
                    // Break time just ran out - transition to "time up" mode
                    await handleBreakTimeUp();
                }
            }
        } else {
            if (!breakOverlay.classList.contains('hidden')) {
                await hideBreakOverlay();
            }
        }
    } catch (error) {
        console.error('Failed to update timer state:', error);
    }
}

// Update UI
// overrideState: optional state to display instead of timerState (used for frozen main window)
function updateUI(overrideState) {
    const displayState = overrideState || timerState;
    if (!displayState) return;

    // Main window elements: use displayState (may be frozen work state)
    const minutes = Math.floor(displayState.remaining_seconds / 60);
    const seconds = displayState.remaining_seconds % 60;
    const timeStr = `${minutes}:${seconds.toString().padStart(2, '0')}`;
    timeDisplay.textContent = timeStr;

    // Phase label for main window
    let mainPhaseName = displayState.phase === 'Work' ? 'Pomodoro' :
        displayState.phase === 'Break' ? 'Break' : 'Long Break';
    phaseLabel.textContent = mainPhaseName;

    // Session count from display state
    sessionCount.textContent = displayState.session_count;

    // Break debt from display state
    const debtMinutes = Math.floor(displayState.break_debt_seconds / 60);
    const debtSeconds = displayState.break_debt_seconds % 60;
    breakDebt.textContent = `Break debt: ${debtMinutes}m ${debtSeconds}s`;

    // Emergency skips display
    const skipsUsed = displayState.emergency_skips_today ?? 0;
    const skipsLimit = displayState.emergency_skips_limit ?? 0;
    const skipsRemaining = Math.max(0, skipsLimit - skipsUsed);
    if (emergencySkips) {
        emergencySkips.textContent = `Emergency skips left: ${skipsRemaining}/${skipsLimit}`;
    }

    // Progress circle for main window
    const mainRemainingRatio = displayState.total_seconds > 0
        ? (displayState.remaining_seconds / displayState.total_seconds)
        : 0;
    const circumference = 2 * Math.PI * 98;
    const mainOffset = circumference * (1 - mainRemainingRatio);
    progressCircle.style.strokeDashoffset = mainOffset;

    // Breakshield elements: always use real timerState for break countdown
    if (IS_BREAKSHIELD && timerState) {
        const breakTimeUp = (timerState.phase === 'Break' || timerState.phase === 'LongBreak') &&
            timerState.status === 'Paused' &&
            timerState.remaining_seconds === 0;

        if (breakTimeUp) {
            // Hide entire break timer, show break time up message at top center
            if (breakTimer) {
                breakTimer.classList.add('hidden');
            }
            breakTimeUpContainer.classList.remove('hidden');
        } else {
            // Show break timer, hide break time up message
            if (breakTimer) {
                breakTimer.classList.remove('hidden');
            }
            breakTimeUpContainer.classList.add('hidden');
            const breakMinutes = Math.floor(timerState.remaining_seconds / 60);
            const breakSeconds = timerState.remaining_seconds % 60;
            const breakTimeStr = `${breakMinutes}:${breakSeconds.toString().padStart(2, '0')}`;
            breakTimeDisplay.textContent = breakTimeStr;
        }

        let breakPhaseName = timerState.phase === 'Work' ? 'Pomodoro' :
            timerState.phase === 'Break' ? 'Break' : 'Long Break';
        breakPhaseLabel.textContent = breakPhaseName;
    }

    // Update pause button based on display state
    if (displayState.status === 'Running') {
        pauseIcon.src = 'pause.svg';
        pauseBtn.title = 'Pause';
    } else if (displayState.status === 'Paused') {
        pauseIcon.src = 'play.svg';
        pauseBtn.title = 'Resume';
    } else {
        pauseIcon.src = 'play.svg';
        pauseBtn.title = 'Start';
    }

    // Enable/disable skip button based on display state
    skipBtn.disabled = displayState.phase !== 'Work';
    skipBtn.style.opacity = displayState.phase === 'Work' ? '1' : '0.5';
}

// Show break overlay
async function showBreakOverlay() {
    // Breakshield window only: Idempotent guard.
    if (!IS_BREAKSHIELD) return;
    if (!breakOverlay.classList.contains('hidden')) return;

    document.body.classList.add('break-active');
    breakOverlay.classList.remove('hidden');

    // Make window fullscreen and always-on-top (and focused) BEFORE grabbing input.
    // If we grab first, the webview may not receive the emergency chord.
    try {
        const { getCurrentWindow } = window.__TAURI__.window;
        const appWindow = getCurrentWindow();
        if (IS_BREAKSHIELD) {
            await appWindow.show();
            await appWindow.setFullscreen(true);
            await appWindow.setAlwaysOnTop(true);
            await appWindow.setFocus();
        }
    } catch (error) {
        console.error('Failed to set window properties:', error);
    }

    // Check if break time is up
    const breakTimeUp = (timerState.phase === 'Break' || timerState.phase === 'LongBreak') &&
        timerState.status === 'Paused' &&
        timerState.remaining_seconds === 0;

    // Only activate input blocking if break is still running
    if (!breakTimeUp) {
        try {
            await invoke('activate_input_blocking');
        } catch (error) {
            console.error('Failed to activate input blocking:', error);
        }

        // Show emergency skip UI if limit not reached
        if (window.checkAndShowEmergencySkip) {
            await window.checkAndShowEmergencySkip();
        }
    } else {
        // Break time is up - deactivate input blocking and set up interaction listener
        try {
            await invoke('deactivate_input_blocking');
        } catch (error) {
            console.error('Failed to deactivate input blocking:', error);
        }

        // Set up one-time listener for any user interaction
        setupBreakCompleteListener();
    }

}

// Hide break overlay
async function hideBreakOverlay() {
    // Breakshield window only: Idempotent guard.
    if (!IS_BREAKSHIELD) return;
    if (breakOverlay.classList.contains('hidden')) return;

    breakOverlay.classList.add('hidden');
    document.body.classList.remove('break-active');

    // Reset break time up flag
    breakTimeUpHandled = false;

    // Hide emergency skip UI
    if (window.hideEmergencySkipUI) {
        window.hideEmergencySkipUI();
    }

    // Remove break complete listener if it exists
    removeBreakCompleteListener();

    // Deactivate input blocking
    try {
        await invoke('deactivate_input_blocking');
    } catch (error) {
        console.error('Failed to deactivate input blocking:', error);
    }

    // Restore window to normal
    try {
        const { getCurrentWindow } = window.__TAURI__.window;
        const appWindow = getCurrentWindow();
        await appWindow.setFullscreen(false);
        await appWindow.setAlwaysOnTop(false);
    } catch (error) {
        console.error('Failed to restore window properties:', error);
    }
}

// Handle transition to break time up state
let breakTimeUpHandled = false;

async function handleBreakTimeUp() {
    // Only run once per break completion
    if (breakTimeUpHandled) {
        console.log('handleBreakTimeUp already handled, skipping');
        return;
    }
    breakTimeUpHandled = true;

    console.log('=== Break time is up ===');
    console.log('Deactivating input blocking and setting up listeners');

    // Hide emergency skip UI
    if (window.hideEmergencySkipUI) {
        window.hideEmergencySkipUI();
        console.log('Emergency skip UI hidden');
    }

    // Deactivate input blocking to allow user interaction
    try {
        await invoke('deactivate_input_blocking');
        console.log('Input blocking deactivated successfully');
    } catch (error) {
        console.error('Failed to deactivate input blocking:', error);
    }

    // Set up one-time listener for any user interaction
    setupBreakCompleteListener();
    
    // Also add a direct click handler to the break overlay as a fallback
    if (breakOverlay) {
        breakOverlay.style.cursor = 'pointer';
        breakOverlay.onclick = () => completeBreakAndDismiss();
    }
    
    console.log('=== Ready for user interaction ===');
}

// Helper function to complete break and dismiss breakshield
async function completeBreakAndDismiss() {
    console.log('Completing break and dismissing breakshield...');
    try {
        await invoke('complete_break');
        console.log('complete_break invoked successfully');
        await invoke('hide_breakshield');
        console.log('hide_breakshield invoked');
        // Close this window
        const { getCurrentWindow } = window.__TAURI__.window;
        const appWindow = getCurrentWindow();
        await appWindow.close();
    } catch (error) {
        console.error('Failed to complete break:', error);
    }
}

// Set up listener for user interaction to complete break
let breakCompleteHandlers = null;

function setupBreakCompleteListener() {
    // Remove any existing listeners first
    removeBreakCompleteListener();

    console.log('Setting up break complete listeners');

    const handleInteraction = async (eventType) => {
        console.log(`User interaction detected (${eventType}), completing break...`);
        
        // Remove listeners immediately to prevent multiple calls
        removeBreakCompleteListener();
        
        await completeBreakAndDismiss();
    };

    // Listen for keyboard and click events only (not mousemove - screen blank can trigger synthetic mouse events)
    const keyHandler = () => handleInteraction('keydown');
    const clickHandler = () => handleInteraction('click');

    // Use capture phase to ensure we get events even if something else is blocking them
    document.addEventListener('keydown', keyHandler, { capture: true });
    document.addEventListener('click', clickHandler, { capture: true });
    window.addEventListener('keydown', keyHandler, { capture: true });
    window.addEventListener('click', clickHandler, { capture: true });

    breakCompleteHandlers = { keyHandler, clickHandler };
    
    console.log('Break complete listeners set up successfully');
}

function removeBreakCompleteListener() {
    if (breakCompleteHandlers) {
        document.removeEventListener('keydown', breakCompleteHandlers.keyHandler, { capture: true });
        document.removeEventListener('click', breakCompleteHandlers.clickHandler, { capture: true });
        window.removeEventListener('keydown', breakCompleteHandlers.keyHandler, { capture: true });
        window.removeEventListener('click', breakCompleteHandlers.clickHandler, { capture: true });
        breakCompleteHandlers = null;
    }
}

// Open settings popup window
async function openSettings() {
    try {
        // Build URL for settings.html
        // In dev mode, use the same base URL; in production, it will be an app:// URL
        const base = window.location.href.replace(/index\.html.*$/, '').replace(/#.*$/, '');
        const settingsUrl = `${base}settings.html`;
        await invoke('show_settings', { url: settingsUrl });
    } catch (error) {
        console.error('Failed to open settings:', error);
    }
}

// Update stats
async function updateStats() {
    if (!timerState) return;

    document.getElementById('sessionsToday').textContent = timerState.session_count;
    document.getElementById('emergencySkipsToday').textContent = timerState.emergency_skips_today;

    const debtMinutes = Math.floor(timerState.break_debt_seconds / 60);
    document.getElementById('breakDebtStat').textContent = `${debtMinutes} minutes`;
}

// Initialize
async function init() {
    // Listen for Tauri events
    const { listen } = window.__TAURI__.event;

    // Listen for phase changes
    listen('phase-changed', (event) => {
        console.log('Phase changed to:', event.payload);
        updateTimerState();
    });

    // Listen for break started
    listen('break-started', async (event) => {
        console.log('Break started:', event.payload);
        // Activate break overlay
        await updateTimerState();
    });

    // Auto-start the timer when app opens (main window only)
    if (!IS_BREAKSHIELD) {
        await invoke('start_timer');
        pauseIcon.src = 'pause.svg';
    } else {
        // Breakshield window: maintain fullscreen when screen unblanks
        document.addEventListener('visibilitychange', async () => {
            if (document.visibilityState === 'visible') {
                try {
                    const { getCurrentWindow } = window.__TAURI__.window;
                    const appWindow = getCurrentWindow();
                    await appWindow.setFullscreen(true);
                    await appWindow.setAlwaysOnTop(true);
                    await appWindow.setFocus();
                } catch (error) {
                    console.error('Failed to restore fullscreen after visibility change:', error);
                }
            }
        });
    }

    await updateTimerState();

    // Update timer every second
    updateInterval = setInterval(async () => {
        await updateTimerState();
    }, 1000);
}

// Wait for Tauri API and DOM to be ready
function waitForTauri() {
    console.log('Waiting for Tauri...', window.__TAURI__);
    if (window.__TAURI__ && window.__TAURI__.core) {
        console.log('Tauri API found, initializing...');
        invoke = window.__TAURI__.core.invoke;
        initDOMElements();
        if (IS_BREAKSHIELD) {
            document.body.classList.add('breakshield');
            createBreakScreen();
        }
        console.log('DOM elements initialized, pauseBtn:', pauseBtn);
        setupEventListeners();
        console.log('Event listeners set up');
        init();
        console.log('Init complete');
    } else {
        console.log('Tauri not ready, retrying...');
        setTimeout(waitForTauri, 100);
    }
}

// Start the app
console.log('Script loaded, document.readyState:', document.readyState);
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', waitForTauri);
} else {
    waitForTauri();
}

