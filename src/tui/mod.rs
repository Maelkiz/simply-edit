//! Full-screen interactive TUI used by commands that prompt for a value on a TTY.
//!
//! Each command implements [`Screen`] (see `screens/`) and hands it to [`run_screen`], which owns
//! the terminal session, layout, and event loop:
//!
//! - `handle_key` is pure state logic and returns an [`Outcome`]: `Redraw` when the preview image
//!   must be rebuilt, `Continue` when only text changed (header, controls and footer are redrawn
//!   every loop regardless), and `Confirm` / `Cancel` to leave. Esc, `q` and Ctrl+C cancel.
//! - `frame` turns the downscaled preview base into the image to show; keep it cheap, since it
//!   runs on every `Redraw`.
//! - `controls` and `hints` fill the side/bottom panel and the key-hint footer; `result_size`
//!   lets the header show how the output dimensions change.
//!
//! `run_screen` returns the confirmed screen, and the caller then applies the chosen operation to
//! the full-resolution image and saves it. To add a command: write a screen in `screens/`, export
//! it from `screens/mod.rs`, and call `run_screen` from the command when stdin is a terminal,
//! keeping a stdin fallback for piped input.

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
