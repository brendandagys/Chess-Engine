use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backward")
        .as_millis() as u64
}
