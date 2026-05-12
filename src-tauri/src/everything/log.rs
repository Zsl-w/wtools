//! Logging utility - forwards to the `log` crate

pub fn log(msg: &str) {
    log::info!("[everything] {}", msg);
}