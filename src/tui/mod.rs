//! Full-screen interactive TUI used by commands that prompt for a value on a TTY.

mod app;
mod terminal;

#[allow(unused_imports)]
pub(crate) use app::{Outcome, Screen, is_cancel_key, run_screen};
