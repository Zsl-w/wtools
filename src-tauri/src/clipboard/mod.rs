pub mod history;
mod monitor;

pub use monitor::{start_clipboard_monitor, stop_clipboard_monitor};
#[allow(unused_imports)]
pub use monitor::is_monitor_running;
