use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_duration_since_unix() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}
