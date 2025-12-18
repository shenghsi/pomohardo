use crate::config::{load_daily_limit_state, save_daily_limit_state, Config, DailyLimitState};
use chrono::{DateTime, Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Hard maximum for emergency skips per day.
// If someone wants to cheat by editing config, they'll cheat anyway — but we still clamp.
const MAX_EMERGENCY_SKIPS_PER_DAY: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    Work,
    Break,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimerStatus {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub phase: Phase,
    pub status: TimerStatus,
    pub remaining_seconds: u32,
    pub total_seconds: u32,
    pub session_count: u32,
    pub break_debt_seconds: u32,
    pub emergency_skips_today: u32,
    pub emergency_skips_limit: u32,
    pub emergency_limit_locked: bool,
}

pub struct TimerEngine {
    phase: Phase,
    status: TimerStatus,
    start_time: Option<DateTime<Utc>>,
    pause_time: Option<DateTime<Utc>>,
    paused_duration: Duration,
    session_count: u32,
    break_debt_seconds: u32,
    emergency_skips_today: u32,
    last_skip_reset: DateTime<Local>,
    limit_locked_date: Option<NaiveDate>,
    locked_emergency_skips_per_day: Option<u32>,
    config: Config,
}

impl TimerEngine {
    pub fn new(config: Config) -> Self {
        let state = load_daily_limit_state();
        let (limit_locked_date, locked_emergency_skips_per_day) = parse_daily_limit_state(&state);
        Self {
            phase: Phase::Work,
            status: TimerStatus::Stopped,
            start_time: None,
            pause_time: None,
            paused_duration: Duration::ZERO,
            session_count: 0,
            break_debt_seconds: 0,
            emergency_skips_today: 0,
            last_skip_reset: Local::now(),
            limit_locked_date,
            locked_emergency_skips_per_day,
            config,
        }
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    fn clamp_emergency_limit(limit: u32) -> u32 {
        limit.min(MAX_EMERGENCY_SKIPS_PER_DAY)
    }

    fn today() -> NaiveDate {
        Local::now().date_naive()
    }

    fn is_limit_locked_today(&self) -> bool {
        self.limit_locked_date == Some(Self::today()) && self.locked_emergency_skips_per_day.is_some()
    }

    fn effective_emergency_limit_per_day(&self) -> u32 {
        if self.is_limit_locked_today() {
            Self::clamp_emergency_limit(self.locked_emergency_skips_per_day.unwrap_or(self.config.emergency_skips_per_day))
        } else {
            Self::clamp_emergency_limit(self.config.emergency_skips_per_day)
        }
    }

    fn maybe_reset_daily_lock(&mut self) {
        if self.limit_locked_date != Some(Self::today()) {
            self.limit_locked_date = None;
            self.locked_emergency_skips_per_day = None;
            let _ = save_daily_limit_state(&DailyLimitState::default());
        }
    }

    pub fn start(&mut self) {
        if self.status == TimerStatus::Stopped {
            self.start_time = Some(Utc::now());
            self.status = TimerStatus::Running;
            self.session_count = 0;
            self.phase = Phase::Work;
            self.paused_duration = Duration::ZERO;
            self.pause_time = None;
        }
    }

    pub fn pause(&mut self) {
        if self.status == TimerStatus::Running {
            self.pause_time = Some(Utc::now());
            self.status = TimerStatus::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.status == TimerStatus::Paused {
            if let Some(pause_time) = self.pause_time {
                let paused = Utc::now().signed_duration_since(pause_time);
                self.paused_duration += paused.to_std().unwrap_or(Duration::ZERO);
            }
            self.pause_time = None;
            self.status = TimerStatus::Running;
        }
    }

    pub fn skip_work(&mut self) -> Result<(), String> {
        if self.phase != Phase::Work {
            return Err("Can only skip work phase".to_string());
        }
        self.transition_to_break();
        Ok(())
    }

    pub fn request_emergency_skip(&mut self, config: &Config) -> Result<bool, String> {
        if !matches!(self.phase, Phase::Break | Phase::LongBreak) {
            return Err("Emergency skip only allowed during breaks".to_string());
        }

        self.reset_daily_skip_count_if_needed();
        self.maybe_reset_daily_lock();

        let limit = self.effective_emergency_limit_per_day();
        if self.emergency_skips_today >= limit {
            return Ok(false); // Denied
        }

        let remaining = self.get_remaining_seconds();
        // Per requirement: emergency skip overwrites debt with the remaining break time.
        // Next break duration = base_break + debt.
        self.break_debt_seconds = remaining;

        // Cap break debt if configured
        if self.break_debt_seconds > config.break_debt_cap_minutes * 60 {
            self.break_debt_seconds = config.break_debt_cap_minutes * 60;
        }

        self.emergency_skips_today += 1;
        // Emergency skip: do NOT clear break debt; we’re explicitly carrying remaining time forward.
        self.transition_to_work(false);
        Ok(true) // Approved
    }

    pub fn get_state(&self) -> TimerState {
        TimerState {
            phase: self.phase,
            status: self.status,
            remaining_seconds: self.get_remaining_seconds(),
            total_seconds: self.get_phase_duration(),
            session_count: self.session_count,
            break_debt_seconds: self.break_debt_seconds,
            emergency_skips_today: self.emergency_skips_today,
            emergency_skips_limit: self.effective_emergency_limit_per_day(),
            emergency_limit_locked: self.is_limit_locked_today(),
        }
    }

    pub fn get_break_debt(&self) -> u32 {
        self.break_debt_seconds
    }

    fn get_phase_duration(&self) -> u32 {
        match self.phase {
            Phase::Work => self.config.work_duration_minutes * 60,
            Phase::Break => {
                self.config.break_duration_minutes * 60 + self.break_debt_seconds
            }
            Phase::LongBreak => {
                self.config.long_break_duration_minutes * 60 + self.break_debt_seconds
            }
        }
    }

    fn get_remaining_seconds(&self) -> u32 {
        if self.status == TimerStatus::Stopped {
            return self.get_phase_duration();
        }

        let start = match self.start_time {
            Some(t) => t,
            None => return self.get_phase_duration(),
        };

        let now = Utc::now();
        let elapsed = now.signed_duration_since(start).to_std().unwrap_or(Duration::ZERO);
        let elapsed = elapsed.checked_sub(self.paused_duration).unwrap_or(Duration::ZERO);

        let phase_duration = Duration::from_secs(self.get_phase_duration() as u64);
        let remaining = phase_duration.checked_sub(elapsed).unwrap_or(Duration::ZERO);

        remaining.as_secs() as u32
    }

    fn transition_to_break(&mut self) {
        // Once the first work session of the day ends (Work -> Break), lock the daily emergency-skip limit.
        self.maybe_reset_daily_lock();
        if !self.is_limit_locked_today() {
            let locked = Self::clamp_emergency_limit(self.config.emergency_skips_per_day);
            self.limit_locked_date = Some(Self::today());
            self.locked_emergency_skips_per_day = Some(locked);
            let _ = save_daily_limit_state(&DailyLimitState {
                locked_date: Some(Self::today().to_string()),
                locked_limit: Some(locked),
            });
        }

        self.session_count += 1;

        if self.session_count >= self.config.sessions_before_long_break {
            self.phase = Phase::LongBreak;
            self.session_count = 0;
        } else {
            self.phase = Phase::Break;
        }

        self.status = TimerStatus::Running;
        self.start_time = Some(Utc::now());
        self.paused_duration = Duration::ZERO;
        self.pause_time = None;
    }

    fn transition_to_work(&mut self, clear_break_debt: bool) {
        self.phase = Phase::Work;
        self.status = TimerStatus::Running;
        if clear_break_debt {
            self.break_debt_seconds = 0;
        }
        self.start_time = Some(Utc::now());
        self.paused_duration = Duration::ZERO;
        self.pause_time = None;
    }

    fn reset_daily_skip_count_if_needed(&mut self) {
        let now = Local::now();
        if now.date_naive() != self.last_skip_reset.date_naive() {
            self.emergency_skips_today = 0;
            self.last_skip_reset = now;
            // New day => unlock the daily emergency limit.
            self.limit_locked_date = None;
            self.locked_emergency_skips_per_day = None;
            let _ = save_daily_limit_state(&DailyLimitState::default());
        }
    }

    /// Check if timer should transition to next phase and perform transition if needed
    /// Returns true if a transition occurred
    pub fn check_and_transition(&mut self) -> bool {
        if self.status != TimerStatus::Running {
            return false;
        }

        self.maybe_reset_daily_lock();

        if self.get_remaining_seconds() == 0 {
            match self.phase {
                Phase::Work => {
                    self.transition_to_break();
                    return true;
                }
                Phase::Break | Phase::LongBreak => {
                    // Break time is up - pause instead of auto-transitioning to work
                    // User interaction will trigger the transition
                    self.status = TimerStatus::Paused;
                    return true;
                }
            }
        }
        false
    }
    
    /// Transition from break to work after user interaction
    /// Clears break debt since break time was completed
    pub fn complete_break(&mut self) {
        if matches!(self.phase, Phase::Break | Phase::LongBreak) && self.status == TimerStatus::Paused {
            self.transition_to_work(true);
        }
    }
}

fn parse_daily_limit_state(state: &DailyLimitState) -> (Option<NaiveDate>, Option<u32>) {
    let Some(date_str) = state.locked_date.as_deref() else {
        return (None, None);
    };
    let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return (None, None);
    };
    // Only keep lock if it's for today
    if date == Local::now().date_naive() {
        (Some(date), state.locked_limit)
    } else {
        (None, None)
    }
}

