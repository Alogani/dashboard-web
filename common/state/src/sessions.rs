use rand::Rng;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TOKEN_LENGTH: usize = 43; // ~256 bits of entropy with alphanumeric characters

struct Session {
    username: String,
    expires_at: Instant,
}

/// In-memory store of authenticated sessions, identified by a random token.
///
/// Sessions are lost on restart, which requires users to log in again.
#[derive(Default)]
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Session>>,
}

impl SessionStore {
    /// Creates a new session for `username` and returns its token
    pub fn create(&self, username: &str, duration: Duration) -> String {
        let token: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(TOKEN_LENGTH)
            .map(char::from)
            .collect();

        let now = Instant::now();
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions.retain(|_, session| session.expires_at > now);
        sessions.insert(
            token.clone(),
            Session {
                username: username.to_string(),
                expires_at: now + duration,
            },
        );
        token
    }

    /// Returns the username associated with a valid (not expired) session token
    pub fn get_username(&self, token: &str) -> Option<String> {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        match sessions.get(token) {
            Some(session) if session.expires_at > Instant::now() => Some(session.username.clone()),
            Some(_) => {
                sessions.remove(token);
                None
            }
            None => None,
        }
    }

    pub fn revoke(&self, token: &str) {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions.remove(token);
    }

    /// Revokes all the sessions of the users for which `predicate` returns true
    pub fn revoke_users<F: Fn(&str) -> bool>(&self, predicate: F) {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        sessions.retain(|_, session| !predicate(&session.username));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let store = SessionStore::default();
        let token = store.create("alice", Duration::from_secs(60));
        assert_eq!(token.len(), TOKEN_LENGTH);
        assert_eq!(store.get_username(&token).as_deref(), Some("alice"));
        assert_eq!(store.get_username("unknown"), None);

        let other = store.create("alice", Duration::from_secs(60));
        assert_ne!(token, other);

        store.revoke(&token);
        assert_eq!(store.get_username(&token), None);
        assert_eq!(store.get_username(&other).as_deref(), Some("alice"));

        store.revoke_users(|username| username == "alice");
        assert_eq!(store.get_username(&other), None);
    }

    #[test]
    fn test_session_expiry() {
        let store = SessionStore::default();
        let token = store.create("alice", Duration::ZERO);
        assert_eq!(store.get_username(&token), None);
    }
}
