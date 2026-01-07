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
    notification_sent: bool,
}

impl TimerEngine {
    pub fn new(config: Config) -> Self {
        let state = load_daily_limit_state();
        let (limit_locked_date, locked_emergency_skips_per_day) = parse_daily_limit_state(&state);

        // Restore daily stats if they belong to today
        let today_str = Self::today().to_string();
        let is_today = state.last_skip_reset_date.as_deref() == Some(&today_str);

        let (session_count, break_debt_seconds, emergency_skips_today, last_skip_reset) =
            if is_today {
                (
                    state.session_count,
                    state.break_debt_seconds,
                    state.emergency_skips_today,
                    Local::now(), // We just update these to now, date logic handles the rest
                )
            } else {
                (0, 0, 0, Local::now())
            };

        Self {
            phase: Phase::Work,
            status: TimerStatus::Stopped,
            start_time: None,
            pause_time: None,
            paused_duration: Duration::ZERO,
            session_count,
            break_debt_seconds,
            emergency_skips_today,
            last_skip_reset,
            limit_locked_date,
            locked_emergency_skips_per_day,
            config,
            notification_sent: false,
        }
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    fn save_state(&self) {
        let state = DailyLimitState {
            locked_date: self.limit_locked_date.map(|d| d.to_string()),
            locked_limit: self.locked_emergency_skips_per_day,
            session_count: self.session_count,
            break_debt_seconds: self.break_debt_seconds,
            emergency_skips_today: self.emergency_skips_today,
            last_skip_reset_date: Some(Self::today().to_string()),
        };
        let _ = save_daily_limit_state(&state);
    }

    fn clamp_emergency_limit(limit: u32) -> u32 {
        limit.min(MAX_EMERGENCY_SKIPS_PER_DAY)
    }

    fn today() -> NaiveDate {
        Local::now().date_naive()
    }

    fn is_limit_locked_today(&self) -> bool {
        self.limit_locked_date == Some(Self::today())
            && self.locked_emergency_skips_per_day.is_some()
    }

    fn effective_emergency_limit_per_day(&self) -> u32 {
        if self.is_limit_locked_today() {
            Self::clamp_emergency_limit(
                self.locked_emergency_skips_per_day
                    .unwrap_or(self.config.emergency_skips_per_day),
            )
        } else {
            Self::clamp_emergency_limit(self.config.emergency_skips_per_day)
        }
    }

    fn maybe_reset_daily_lock(&mut self) {
        if self.limit_locked_date != Some(Self::today()) {
            self.limit_locked_date = None;
            self.locked_emergency_skips_per_day = None;
            self.save_state();
        }
    }

    pub fn start(&mut self) {
        if self.status == TimerStatus::Stopped {
            self.reset_daily_stats_if_needed();
            self.start_time = Some(Utc::now());
            self.status = TimerStatus::Running;
            // Don't reset session_count here, it might be restored or continuing today's work
            // Only reset if it was specifically a "new run" logic, but typically we want to keep today's count.
            // However, original logic was `self.session_count = 0;`.
            // If we want persistence, we should probably NOT reset it on manual start unless it's a new day.
            // But if the user pressed "Start" from a user-initiated Stop state, maybe they intend to reset?
            // Standard pomodoro apps usually keep the daily count.
            // We will trust the loaded state.
            if !self.is_limit_locked_today() && self.session_count == 0 {
                // fresh start
            }

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

        self.reset_daily_stats_if_needed();
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
        // transition_to_work saves state, but let's ensure we save the skip count update too
        // actually transition_to_work calls save_state, so we are good.
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
            Phase::Break => self.config.break_duration_minutes * 60 + self.break_debt_seconds,
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
        let elapsed = now
            .signed_duration_since(start)
            .to_std()
            .unwrap_or(Duration::ZERO);

        let mut total_paused = self.paused_duration;
        if self.status == TimerStatus::Paused {
            if let Some(pause_time) = self.pause_time {
                let current_pause = now
                    .signed_duration_since(pause_time)
                    .to_std()
                    .unwrap_or(Duration::ZERO);
                total_paused += current_pause;
            }
        }

        let elapsed = elapsed.checked_sub(total_paused).unwrap_or(Duration::ZERO);

        let phase_duration = Duration::from_secs(self.get_phase_duration() as u64);
        let remaining = phase_duration
            .checked_sub(elapsed)
            .unwrap_or(Duration::ZERO);

        remaining.as_secs() as u32
    }

    fn transition_to_break(&mut self) {
        // Once the first work session of the day ends (Work -> Break), lock the daily emergency-skip limit.
        self.maybe_reset_daily_lock();
        if !self.is_limit_locked_today() {
            let locked = Self::clamp_emergency_limit(self.config.emergency_skips_per_day);
            self.limit_locked_date = Some(Self::today());
            self.locked_emergency_skips_per_day = Some(locked);
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
        self.notification_sent = false; // Reset for next work session

        self.save_state();
    }

    fn transition_to_work(&mut self, clear_break_debt: bool) {
        self.reset_daily_stats_if_needed();
        self.phase = Phase::Work;
        self.status = TimerStatus::Running;
        if clear_break_debt {
            self.break_debt_seconds = 0;
        }
        self.start_time = Some(Utc::now());
        self.paused_duration = Duration::ZERO;
        self.pause_time = None;
        self.notification_sent = false; // Reset for new work session

        self.save_state();
    }

    fn reset_daily_stats_if_needed(&mut self) {
        let now = Local::now();
        if now.date_naive() != self.last_skip_reset.date_naive() {
            self.emergency_skips_today = 0;
            self.session_count = 0;
            self.break_debt_seconds = 0;
            self.last_skip_reset = now;
            // New day => unlock the daily emergency limit.
            self.limit_locked_date = None;
            self.locked_emergency_skips_per_day = None;
            self.save_state();
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
        if matches!(self.phase, Phase::Break | Phase::LongBreak)
            && self.status == TimerStatus::Paused
        {
            self.transition_to_work(true);
        }
    }
    
    /// Check if notification should be sent (1 minute before work session ends)
    /// Returns true if notification should be sent now
    pub fn should_send_notification(&mut self) -> bool {
        // Only send notification during work phase
        if self.phase != Phase::Work || self.status != TimerStatus::Running {
            return false;
        }
        
        // Only if the feature is enabled
        if !self.config.notify_before_work_end {
            return false;
        }
        
        // Only send once per work session
        if self.notification_sent {
            return false;
        }
        
        let remaining = self.get_remaining_seconds();
        
        // Send notification when exactly 60 seconds (1 minute) remain
        // We check for <= 61 to account for the 1-second polling interval
        if remaining <= 61 && remaining > 59 {
            self.notification_sent = true;
            return true;
        }
        
        false
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_pause_resume_timer_calculation() {
        // Setup
        let config = Config::default();
        let mut timer = TimerEngine::new(config);

        // Start timer
        timer.start();

        // Wait 2 seconds (simulated or real)
        thread::sleep(Duration::from_secs(2));

        // Pause
        timer.pause();
        let remaining_at_pause = timer.get_remaining_seconds();

        // Wait 2 seconds while paused
        thread::sleep(Duration::from_secs(2));

        // Check remaining seconds should be same as when paused (approx)
        let remaining_after_wait = timer.get_remaining_seconds();

        // There might be slight diff due to execution time, but it shouldn't be ~2s diff
        // With the bug, remaining_after_wait would be ~ remaining_at_pause - 2
        // With the fix, remaining_after_wait should be == remaining_at_pause

        assert_eq!(
            remaining_after_wait, remaining_at_pause,
            "Timer should not count down while paused"
        );

        // Resume
        timer.resume();
        thread::sleep(Duration::from_secs(1));

        let remaining_after_resume = timer.get_remaining_seconds();
        assert!(
            remaining_after_resume < remaining_after_wait,
            "Timer should resume counting"
        );
    }
}
