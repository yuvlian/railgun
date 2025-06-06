use bcrypt::{BcryptError, DEFAULT_COST, hash};

pub fn hash_password(password: &str) -> Result<String, BcryptError> {
    hash(password, DEFAULT_COST)
}
