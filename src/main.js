// Wait for Tauri API to be available
let invoke;

// State
let timerState = null;
let config = null;
let updateInterval = null;

// DOM Elements (will be initialized after DOM is ready)
let tabs, tabContents, timeDisplay, phaseLabel, sessionCount, breakDebt;
let progressCircle, pauseBtn, pauseIcon, skipBtn, settingsBtn;
let settingsModal, closeSettings, saveSettings, breakOverlay;
let breakTimeDisplay, breakPhaseLabel, breakProgressCircle, emergencySkipBtn;

// Initialize DOM elements
function initDOMElements() {
    tabs = document.querySelectorAll('.tab');
    tabContents = document.querySelectorAll('.tab-content');
    timeDisplay = document.getElementById('timeDisplay');
    phaseLabel = document.getElementById('phaseLabel');
    sessionCount = document.getElementById('sessionCount');
    breakDebt = document.getElementById('breakDebt');
    progressCircle = document.getElementById('progressCircle');
    pauseBtn = document.getElementById('pauseBtn');
    pauseIcon = document.getElementById('pauseIcon');
    skipBtn = document.getElementById('skipBtn');
    settingsBtn = document.getElementById('settingsBtn');
    settingsModal = document.getElementById('settingsModal');
    closeSettings = document.getElementById('closeSettings');
    saveSettings = document.getElementById('saveSettings');
    breakOverlay = document.getElementById('breakOverlay');
    breakTimeDisplay = document.getElementById('breakTimeDisplay');
    breakPhaseLabel = document.getElementById('breakPhaseLabel');
    breakProgressCircle = document.getElementById('breakProgressCircle');
    emergencySkipBtn = document.getElementById('emergencySkipBtn');
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

    // Settings modal
    settingsBtn.addEventListener('click', () => {
        openSettings();
    });

    closeSettings.addEventListener('click', () => {
        settingsModal.classList.remove('active');
    });

    saveSettings.addEventListener('click', async () => {
        await saveConfig();
        settingsModal.classList.remove('active');
    });

    // Control buttons
    pauseBtn.addEventListener('click', async () => {
        if (!timerState) return;
        
        if (timerState.status === 'Running') {
            await invoke('pause_timer');
            pauseIcon.textContent = '▶';
        } else if (timerState.status === 'Paused') {
            await invoke('resume_timer');
            pauseIcon.textContent = '⏸';
        } else {
            await invoke('start_timer');
            pauseIcon.textContent = '⏸';
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

    emergencySkipBtn.addEventListener('click', async () => {
        if (confirm('Emergency skip will add remaining break time to your next break. Continue?')) {
            try {
                const approved = await invoke('request_emergency_skip');
                if (!approved) {
                    alert('Emergency skip limit reached for today.');
                } else {
                    await updateTimerState();
                }
            } catch (error) {
                console.error('Emergency skip failed:', error);
            }
        }
    });
}

// Update timer state
async function updateTimerState() {
    try {
        timerState = await invoke('get_timer_state');
        updateUI();
        
        // Show/hide break overlay
        if (timerState.phase === 'Break' || timerState.phase === 'LongBreak') {
            if (timerState.status === 'Running') {
                showBreakOverlay();
            }
        } else {
            hideBreakOverlay();
        }
    } catch (error) {
        console.error('Failed to update timer state:', error);
    }
}

// Update UI
function updateUI() {
    if (!timerState) return;
    
    // Update time display
    const minutes = Math.floor(timerState.remaining_seconds / 60);
    const seconds = timerState.remaining_seconds % 60;
    const timeStr = `${minutes}:${seconds.toString().padStart(2, '0')}`;
    timeDisplay.textContent = timeStr;
    breakTimeDisplay.textContent = timeStr;
    
    // Update phase label
    let phaseName = timerState.phase === 'Work' ? 'Pomodoro' : 
                    timerState.phase === 'Break' ? 'Break' : 'Long Break';
    phaseLabel.textContent = phaseName;
    breakPhaseLabel.textContent = phaseName;
    
    // Update session count
    sessionCount.textContent = timerState.session_count;
    
    // Update break debt
    const debtMinutes = Math.floor(timerState.break_debt_seconds / 60);
    const debtSeconds = timerState.break_debt_seconds % 60;
    breakDebt.textContent = `Break debt: ${debtMinutes}m ${debtSeconds}s`;
    
    // Update progress circle
    const progress = timerState.total_seconds > 0 ? 
        1 - (timerState.remaining_seconds / timerState.total_seconds) : 0;
    const circumference = 2 * Math.PI * 90;
    const offset = circumference * (1 - progress);
    progressCircle.style.strokeDashoffset = offset;
    breakProgressCircle.style.strokeDashoffset = offset;
    
    // Update pause button
    if (timerState.status === 'Running') {
        pauseIcon.textContent = '⏸';
    } else if (timerState.status === 'Paused') {
        pauseIcon.textContent = '▶';
    } else {
        pauseIcon.textContent = '▶';
    }
    
    // Enable/disable skip button
    skipBtn.disabled = timerState.phase !== 'Work';
    skipBtn.style.opacity = timerState.phase === 'Work' ? '1' : '0.5';
}

// Show break overlay
function showBreakOverlay() {
    breakOverlay.classList.remove('hidden');
    // Request notification permission if not granted
    if ('Notification' in window && Notification.permission === 'default') {
        Notification.requestPermission();
    }
}

// Hide break overlay
function hideBreakOverlay() {
    breakOverlay.classList.add('hidden');
}

// Open settings
async function openSettings() {
    try {
        config = await invoke('get_config');
        document.getElementById('workDuration').value = config.work_duration_minutes;
        document.getElementById('breakDuration').value = config.break_duration_minutes;
        document.getElementById('longBreakDuration').value = config.long_break_duration_minutes;
        document.getElementById('sessionsBeforeLongBreak').value = config.sessions_before_long_break;
        document.getElementById('emergencySkipsPerDay').value = config.emergency_skips_per_day;
        document.getElementById('breakDebtCap').value = config.break_debt_cap_minutes;
        settingsModal.classList.add('active');
    } catch (error) {
        console.error('Failed to load config:', error);
    }
}

// Save config
async function saveConfig() {
    const newConfig = {
        work_duration_minutes: parseInt(document.getElementById('workDuration').value),
        break_duration_minutes: parseInt(document.getElementById('breakDuration').value),
        long_break_duration_minutes: parseInt(document.getElementById('longBreakDuration').value),
        sessions_before_long_break: parseInt(document.getElementById('sessionsBeforeLongBreak').value),
        emergency_skips_per_day: parseInt(document.getElementById('emergencySkipsPerDay').value),
        break_debt_cap_minutes: parseInt(document.getElementById('breakDebtCap').value),
    };
    
    try {
        await invoke('update_config', { newConfig });
        config = newConfig;
        alert('Settings saved!');
    } catch (error) {
        console.error('Failed to save config:', error);
        alert('Failed to save settings');
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

