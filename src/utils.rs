use std::time::{Duration, Instant};

pub fn is_timeout_expired(start: Instant, timeout: Duration) -> bool {
    Instant::now().duration_since(start) > timeout
}