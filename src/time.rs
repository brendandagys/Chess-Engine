use std::time::{Duration, Instant};

use crate::constants::{
    DEFAULT_PLAYER_INCREMENT_MS, DEFAULT_PLAYER_TIME_REMAINING_MS, SOFT_TO_HARD_LIMIT_RATIO,
};

#[derive(Debug, Clone, Copy)]
pub struct TimeManager {
    pub start_time: Instant,
    pub soft_limit: Duration,
    pub hard_limit: Duration,
    pub fixed_time: bool,
    pub stopped: bool,
}

/// For ease of use in tests
impl Default for TimeManager {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            soft_limit: Duration::from_millis(DEFAULT_PLAYER_TIME_REMAINING_MS),
            hard_limit: Duration::from_millis(DEFAULT_PLAYER_TIME_REMAINING_MS),
            fixed_time: false,
            stopped: false,
        }
    }
}

impl TimeManager {
    /// Create a new TimeManager for each search
    pub fn new(
        fixed_time: bool,
        movetime: Option<u64>, // Explicit per-move time in ms
        wtime: Option<u64>,    // Remaining white time (ms)
        btime: Option<u64>,    // Remaining black time (ms)
        winc: Option<u64>,     // White increment (ms)
        binc: Option<u64>,     // Black increment (ms)
        is_white_turn: bool,
    ) -> Self {
        let start_time = Instant::now();

        let max_search_duration_ms = match fixed_time {
            true => movetime.unwrap_or(1000),
            false => {
                let (time_left, increment) = match is_white_turn {
                    true => (
                        wtime.unwrap_or(DEFAULT_PLAYER_TIME_REMAINING_MS),
                        winc.unwrap_or(DEFAULT_PLAYER_INCREMENT_MS),
                    ),
                    false => (
                        btime.unwrap_or(DEFAULT_PLAYER_TIME_REMAINING_MS),
                        binc.unwrap_or(DEFAULT_PLAYER_INCREMENT_MS),
                    ),
                };

                // Use 1/30 of remaining time + increment, but never more than 25% of total
                (time_left / 30 + increment).min(time_left / 4)
            }
        };

        let (soft_limit_ms, hard_limit_ms) = match fixed_time {
            true => (max_search_duration_ms, max_search_duration_ms),
            false => {
                // e.g. Stop starting new depths at 75% of the budget
                let soft = (max_search_duration_ms as f64 * SOFT_TO_HARD_LIMIT_RATIO) as u64;
                (soft, max_search_duration_ms)
            }
        };

        Self {
            start_time,
            soft_limit: Duration::from_millis(soft_limit_ms),
            hard_limit: Duration::from_millis(hard_limit_ms),
            fixed_time,
            stopped: false,
        }
    }

    /// Time since search began
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if we should stop the current iteration
    pub fn is_soft_limit_reached(&self) -> bool {
        !self.fixed_time && self.elapsed() >= self.soft_limit
    }

    /// Check if we must abort immediately
    pub fn is_hard_limit_reached(&self) -> bool {
        self.elapsed() >= self.hard_limit
    }
}
