//! Full-screen interactive TUI used by commands that prompt for a value on a TTY.

mod app;
mod preview_source;
mod terminal;
mod widgets;

#[allow(unused_imports)]
pub(crate) use app::{Outcome, Screen, is_cancel_key, run_screen};
