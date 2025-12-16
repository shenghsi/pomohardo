use crate::config::Config;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    config: Config,
}

impl TimerEngine {
    pub fn new(config: Config) -> Self {
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
            config,
        }
    }

    pub fn start(&mut self) {
        if self.status == TimerStatus::Stopped {
            self.start_time = Some(Utc::now());
            self.status = TimerStatus::Running;
            self.session_count = 0;
            self.phase = Phase::Work;
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

        if self.emergency_skips_today >= config.emergency_skips_per_day {
            return Ok(false); // Denied
        }

        let remaining = self.get_remaining_seconds();
        self.break_debt_seconds += remaining;

        // Cap break debt if configured
        if self.break_debt_seconds > config.break_debt_cap_minutes * 60 {
            self.break_debt_seconds = config.break_debt_cap_minutes * 60;
        }

        self.emergency_skips_today += 1;
        self.transition_to_work();
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
        self.session_count += 1;

        if self.session_count >= self.config.sessions_before_long_break {
            self.phase = Phase::LongBreak;
            self.session_count = 0;
        } else {
            self.phase = Phase::Break;
        }

        self.start_time = Some(Utc::now());
        self.paused_duration = Duration::ZERO;
        self.pause_time = None;
    }

    fn transition_to_work(&mut self) {
        self.phase = Phase::Work;
        // Clear break debt only if break was completed normally
        if self.get_remaining_seconds() == 0 {
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
        }
    }
}

