use bcrypt::{hash, DEFAULT_COST};

// Utility function to generate a password hash
pub fn generate_password_hash(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}
