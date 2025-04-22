use local_lru::LocalCache;
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple in-memory per-IP rate limiter
///
/// This rate limiter tracks the last request time for each IP address
/// and allows configuring the minimum time between requests.
#[derive(Clone)]
pub struct RateLimiter {
    requests: LocalCache,
    rate_limit_ms: Option<u64>,
}

impl RateLimiter {
    pub fn new(rate_limit_ms: Option<u64>) -> Self {
        Self {
            requests: LocalCache::initialize(1000, 60),
            rate_limit_ms,
        }
    }

    /// Checks if a request from the given IP should be rate limited
    pub fn check_rate_limit(&self, ip: &str) -> bool {
        // If rate limiting is disabled, always allow requests
        if self.rate_limit_ms.is_none() {
            return true;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            * 1000; // Convert to milliseconds

        if let Some(last_bytes) = self.requests.get_item(ip) {
            let last_vec: Vec<u8> = last_bytes.to_vec();
            if last_vec.len() >= 8 {
                let last = u64::from_be_bytes(last_vec[..8].try_into().unwrap());
                let elapsed_ms = now - last;
                if elapsed_ms < self.rate_limit_ms.unwrap() {
                    return false;
                }
            }
        }

        self.requests
            .add_item(ip, now.to_be_bytes().to_vec().into());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_with_limit() {
        let limiter = RateLimiter::new(Some(2000)); // 2 seconds
        let ip = "127.0.0.1";

        // First request should be allowed
        assert!(limiter.check_rate_limit(ip));

        // Second immediate request should be rate limited
        assert!(!limiter.check_rate_limit(ip));
    }

    #[test]
    fn test_rate_limiter_disabled() {
        let limiter = RateLimiter::new(None);
        let ip = "127.0.0.1";

        // All requests should be allowed when rate limiting is disabled
        assert!(limiter.check_rate_limit(ip));
        assert!(limiter.check_rate_limit(ip));
        assert!(limiter.check_rate_limit(ip));
    }
}
