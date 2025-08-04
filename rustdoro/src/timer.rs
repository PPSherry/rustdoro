use std::time::{Duration, Instant};
use crate::config::Config;

/// Session types for the Pomodoro timer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionType {
    /// Get the display text for the session type
    pub fn display_text(&self) -> &'static str {
        match self {
            SessionType::Work => "Work Session",
            SessionType::ShortBreak => "Short Break",
            SessionType::LongBreak => "Long Break",
        }
    }

    /// Get the emoji representation for the session type
    pub fn emoji(&self) -> &'static str {
        match self {
            SessionType::Work => "🍅",
            SessionType::ShortBreak => "☕",
            SessionType::LongBreak => "🏖️",
        }
    }
}

/// Timer states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    /// Timer is currently running
    Running,
    /// Timer is paused
    Paused,
    /// Timer is stopped (initial state or session ended)
    Stopped,
}

/// Main timer structure that manages Pomodoro session state
#[derive(Debug)]
pub struct Timer {
    /// Current session type
    pub current_session: SessionType,
    /// Remaining time in the current session
    pub remaining_time: Duration,
    /// Current state of the timer
    pub state: TimerState,
    /// Duration for work sessions
    pub work_duration: Duration,
    /// Duration for short breaks
    pub short_break_duration: Duration,
    /// Duration for long breaks
    pub long_break_duration: Duration,
    /// Number of completed pomodoros
    pub pomodoros_completed: u32,
    /// Last time the timer was updated (for precise timing)
    pub last_update_time: Option<Instant>,
    /// Count of short breaks taken (used to determine long breaks)
    pub break_count: u8,
    /// Number of pomodoros before a long break
    pub long_break_after_pomodoros: u8,
}

impl Timer {
    /// Create a new timer instance with the given configuration
    pub fn new(config: Config) -> Self {
        let work_duration = Duration::from_secs(config.work_duration_minutes * 60);
        
        Self {
            current_session: SessionType::Work,
            remaining_time: work_duration,
            state: TimerState::Stopped,
            work_duration,
            short_break_duration: Duration::from_secs(config.short_break_duration_minutes * 60),
            long_break_duration: Duration::from_secs(config.long_break_duration_minutes * 60),
            pomodoros_completed: 0,
            last_update_time: None,
            break_count: 0,
            long_break_after_pomodoros: config.long_break_after_pomodoros,
        }
    }

    /// Start or resume the current session timer
    pub fn start(&mut self) {
        self.state = TimerState::Running;
        self.last_update_time = Some(Instant::now());
    }

    /// Pause the current session timer
    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
            self.last_update_time = None;
        }
    }

    /// Resume the paused timer
    pub fn resume(&mut self) {
        if self.state == TimerState::Paused {
            self.start();
        }
    }

    /// Toggle between running and paused states
    pub fn toggle_pause(&mut self) {
        match self.state {
            TimerState::Running => self.pause(),
            TimerState::Paused => self.resume(),
            TimerState::Stopped => self.start(),
        }
    }

    /// Skip the current session and move to the next one
    pub fn skip_session(&mut self) -> bool {
        self.remaining_time = Duration::ZERO;
        self.complete_session()
    }

    /// Update the timer state (should be called regularly, e.g., every second)
    pub fn tick(&mut self) -> bool {
        if self.state != TimerState::Running {
            return false;
        }

        let now = Instant::now();
        if let Some(last_update) = self.last_update_time {
            let elapsed = now.duration_since(last_update);
            
            if self.remaining_time <= elapsed {
                self.remaining_time = Duration::ZERO;
                self.last_update_time = Some(now);
                return self.complete_session();
            } else {
                self.remaining_time -= elapsed;
                self.last_update_time = Some(now);
            }
        }

        false
    }

    /// Complete the current session and transition to the next one
    fn complete_session(&mut self) -> bool {
        let session_completed = true;
        
        match self.current_session {
            SessionType::Work => {
                self.pomodoros_completed += 1;
                
                // Determine if it's time for a long break
                if self.pomodoros_completed % self.long_break_after_pomodoros as u32 == 0 {
                    self.current_session = SessionType::LongBreak;
                    self.remaining_time = self.long_break_duration;
                    self.break_count = 0; // Reset break count after long break
                } else {
                    self.current_session = SessionType::ShortBreak;
                    self.remaining_time = self.short_break_duration;
                    self.break_count += 1;
                }
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.current_session = SessionType::Work;
                self.remaining_time = self.work_duration;
            }
        }

        self.state = TimerState::Stopped;
        self.last_update_time = None;
        session_completed
    }

    /// Get the formatted display time (MM:SS)
    pub fn get_display_time(&self) -> String {
        let total_seconds = self.remaining_time.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Get the current session type
    pub fn get_session_type(&self) -> SessionType {
        self.current_session
    }

    /// Get the number of completed pomodoros
    pub fn get_pomodoros_completed(&self) -> u32 {
        self.pomodoros_completed
    }

    /// Get the current timer state
    pub fn get_state(&self) -> TimerState {
        self.state
    }

    /// Check if the timer is currently running
    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    /// Check if the timer is paused
    pub fn is_paused(&self) -> bool {
        self.state == TimerState::Paused
    }

    /// Check if the timer is stopped
    pub fn is_stopped(&self) -> bool {
        self.state == TimerState::Stopped
    }

    /// Get the progress percentage of the current session (0.0 to 1.0)
    pub fn get_progress(&self) -> f64 {
        let total_duration = match self.current_session {
            SessionType::Work => self.work_duration,
            SessionType::ShortBreak => self.short_break_duration,
            SessionType::LongBreak => self.long_break_duration,
        };

        let elapsed = total_duration - self.remaining_time;
        elapsed.as_secs_f64() / total_duration.as_secs_f64()
    }

    /// Reset the timer to initial state
    pub fn reset(&mut self) {
        self.current_session = SessionType::Work;
        self.remaining_time = self.work_duration;
        self.state = TimerState::Stopped;
        self.pomodoros_completed = 0;
        self.last_update_time = None;
        self.break_count = 0;
    }

    /// Get total session duration for current session type
    pub fn get_total_duration(&self) -> Duration {
        match self.current_session {
            SessionType::Work => self.work_duration,
            SessionType::ShortBreak => self.short_break_duration,
            SessionType::LongBreak => self.long_break_duration,
        }
    }
} 