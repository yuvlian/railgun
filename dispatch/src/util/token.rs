use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const EXPIRE_AFTER_MINUTES: u64 = 2;

pub fn generate_token() -> String {
    let mut token_bytes = [0u8; 16];
    let r1 = fastrand::u64(..);
    let r2 = fastrand::u64(..);
    token_bytes[..8].copy_from_slice(&r1.to_ne_bytes());
    token_bytes[8..].copy_from_slice(&r2.to_ne_bytes());

    let mut token = String::with_capacity(32);
    for byte in &token_bytes {
        write!(token, "{:02x}", byte).unwrap();
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let expiry_minutes = now.as_secs() / 60 + EXPIRE_AFTER_MINUTES;

    format!("{}+{}", token, expiry_minutes)
}

pub fn should_refresh_token(token: &str) -> Result<bool, &'static str> {
    let parts: Vec<&str> = token.split('+').collect();
    if parts.len() != 2 {
        return Err("Invalid token format");
    }
    let Ok(expiry_minutes) = parts[1].parse::<u64>() else {
        return Err("Invalid token format");
    };
    let current_minutes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 60;
    Ok(current_minutes >= expiry_minutes)
}
