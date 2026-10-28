use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Above this number of tracked IPs, the entries older than the delay are purged
const PURGE_THRESHOLD: usize = 1000;

/// A simple in-memory per-IP rate limiter
///
/// This rate limiter tracks the last request time (and the attempt count) for each IP address.
/// Its state is shared between all its clones, but not with other rate limiters.
#[derive(Clone)]
pub struct RateLimiter {
    // IP -> (last attempt in ms, attempt count)
    requests: Arc<Mutex<HashMap<String, (u64, u64)>>>,
    delay_ms: Option<u64>,
    max_attempts: Option<u64>,
}

impl RateLimiter {
    pub fn new(delay_ms: Option<u64>, max_attempts: Option<u64>) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
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
            .unwrap_or_default()
            .as_millis() as u64;

        let mut requests = self.requests.lock().unwrap_or_else(|e| e.into_inner());
        if requests.len() >= PURGE_THRESHOLD {
            requests.retain(|_, (last_attempt, _)| now.saturating_sub(*last_attempt) < delay_ms);
        }

        let (result, attempt_count) = match requests.get(ip).copied() {
            Some((last_attempt, attempt_count)) => {
                // Saturating in case the system clock went backward
                let elapsed_ms = now.saturating_sub(last_attempt);

                match self.max_attempts {
                    // RateLimiter-like behavior: only check time elapsed
                    None => (elapsed_ms >= delay_ms, 0), // Attempt count doesn't matter

                    // AttemptLimiter behavior: allow at most `max` attempts per delay
                    Some(max) => {
                        if elapsed_ms < delay_ms && attempt_count >= max {
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

        requests.insert(ip.to_string(), (now, attempt_count));
        result
    }

    /// Clears the attempt limit entry for the given IP
    pub fn clear(&self, ip: &str) {
        let mut requests = self.requests.lock().unwrap_or_else(|e| e.into_inner());
        requests.remove(ip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempt_limiter() {
        let limiter = RateLimiter::new(Some(60_000), Some(3));
        assert!(limiter.check_limit("1.1.1.1"));
        assert!(limiter.check_limit("1.1.1.1"));
        assert!(limiter.check_limit("1.1.1.1"));
        assert!(!limiter.check_limit("1.1.1.1"));
        assert!(limiter.check_limit("2.2.2.2"));

        limiter.clear("1.1.1.1");
        assert!(limiter.check_limit("1.1.1.1"));
    }

    #[test]
    fn test_limiters_are_independent_and_shared_across_threads() {
        let login_limiter = RateLimiter::new(Some(60_000), Some(2));
        let other_limiter = RateLimiter::new(Some(60_000), None);

        let clone = login_limiter.clone();
        std::thread::spawn(move || {
            assert!(clone.check_limit("1.1.1.1"));
            assert!(clone.check_limit("1.1.1.1"));
        })
        .join()
        .unwrap();

        assert!(other_limiter.check_limit("1.1.1.1"));
        assert!(!login_limiter.check_limit("1.1.1.1"));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(Some(60_000), None);
        assert!(limiter.check_limit("1.1.1.1"));
        assert!(!limiter.check_limit("1.1.1.1"));

        let no_limit = RateLimiter::new(None, None);
        assert!(no_limit.check_limit("1.1.1.1"));
        assert!(no_limit.check_limit("1.1.1.1"));
    }
}
