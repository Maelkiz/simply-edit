pub mod convert;
pub mod transforms;

use indicatif::{ProgressBar, ProgressStyle};
use std::io::{IsTerminal, stderr};
use std::time::Duration;

pub(crate) fn start_spinner(message: &str) -> Option<ProgressBar> {
    if !stderr().is_terminal() {
        return None;
    }

    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{spinner} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_chars("-\\|/ ");

    pb.set_style(style);
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    Some(pb)
}
