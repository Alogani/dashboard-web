use local_lru::LocalCache;
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple in-memory per-IP rate limiter
///
/// This rate limiter tracks the last request time for each IP address
#[derive(Clone)]
pub struct RateLimiter {
    requests: LocalCache,
    delay_ms: Option<u64>,
    max_attempts: Option<u64>,
}

impl RateLimiter {
    pub fn new(delay_ms: Option<u64>, max_attempts: Option<u64>) -> Self {
        Self {
            requests: LocalCache::initialize(1000, 60),
            delay_ms,
            max_attempts,
        }
    }

    /// Checks if a request from the given IP should be rate limited
    pub fn check_limit(&self, ip: &str) -> bool {
        if self.delay_ms.is_none() {
            return true;
        }
        let delay_ms = self.delay_ms.unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            * 1000; // Convert to milliseconds

        let (result, attempt_count) = match self.requests.get_struct::<(u64, u64)>(ip) {
            Some((last_attempt, attempt_count)) => {
                let elapsed_ms = now - last_attempt;

                match self.max_attempts {
                    // RateLimiter-like behavior: only check time elapsed
                    None => (elapsed_ms >= delay_ms, 0), // Attempt count doesn't matter

                    // AttemptLimiter behavior: check attempts and time
                    Some(max) => {
                        if elapsed_ms < delay_ms && attempt_count > max {
                            (false, attempt_count + 1)
                        } else if elapsed_ms >= delay_ms {
                            (true, 1)
                        } else {
                            (true, attempt_count + 1)
                        }
                    }
                }
            }
            None => (true, self.max_attempts.is_some() as u64), // Start at 0 or 1 based on mode
        };

        self.requests
            .add_struct::<(u64, u64)>(ip, (now, attempt_count));
        result
    }

    /// Clears the attempt limit entry for the given IP
    pub fn clear(&self, ip: &str) {
        if let Some(_) = self.requests.get_item(ip) {
            self.requests.add_struct(ip, (0, 0));
        }
    }
}
