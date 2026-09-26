//! Full-screen interactive TUI used by commands that prompt for a value on a TTY.

mod app;
mod preview_source;
pub(crate) mod screens;
mod terminal;
mod widgets;

pub(crate) use app::{Outcome, Screen, is_cancel_key, run_screen};

/// Tell the user a cancelled screen left their files untouched.
pub(crate) fn print_cancelled() {
    eprintln!("Cancelled — nothing saved.");
}
