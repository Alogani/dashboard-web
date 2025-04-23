use local_lru::LocalCache;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Deserialize, Serialize)]
struct RateLimitEntry<T> {
    last_attempt: u64,
    data: T,
}

/// A simple in-memory per-IP rate limiter
///
/// This rate limiter tracks the last request time for each IP address
/// and allows configuring the minimum time between requests.
#[derive(Clone)]
pub struct RateLimiter<T>
where
    T: Default,
{
    requests: LocalCache,
    rate_limit_ms: Option<u64>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> RateLimiter<T>
where
    T: Default,
{
    pub fn new(rate_limit_ms: Option<u64>) -> Self {
        Self {
            requests: LocalCache::initialize(1000, 60),
            rate_limit_ms,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Checks if a request from the given IP should be rate limited
    pub fn check_rate_limit(&self, ip: &str, predicate: fn(bool, T) -> (bool, T)) -> bool {
        if self.rate_limit_ms.is_none() {
            return true;
        }
        let rate_limit_ms = self.rate_limit_ms.unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            * 1000; // Convert to milliseconds

        let (result, new_data) = match self.requests.get_struct::<RateLimitEntry<T>>(ip) {
            Some(entry) => {
                let elapsed_ms = now - entry.last_attempt;
                predicate(elapsed_ms < rate_limit_ms, entry.data)
            }
            None => predicate(true, T::default()),
        };

        self.requests.add_struct(
            ip,
            RateLimitEntry {
                last_attempt: now,
                data: new_data,
            },
        );
        result
    }

    /// Clears the rate limit entry for the given IP
    pub fn clear(&self, ip: &str) {
        if let Some(_) = self.requests.get_item(ip) {
            self.requests.add_struct(
                ip,
                RateLimitEntry {
                    last_attempt: 0,
                    data: T::default(),
                },
            );
        }
    }
}
