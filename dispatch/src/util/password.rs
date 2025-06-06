use bcrypt::{BcryptError, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, BcryptError> {
    bcrypt::hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, BcryptError> {
    bcrypt::verify(password, hash)
}
