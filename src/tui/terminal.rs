use ratatui::DefaultTerminal;

/// RAII guard for the full-screen terminal.
///
/// Entering switches to the alternate screen and raw mode, and installs a panic hook that restores
/// the terminal before the panic message prints. Dropping the guard restores the terminal, so the
/// user's scrollback is left exactly as it was before the TUI started.
pub(crate) struct TuiSession {
    pub(crate) terminal: DefaultTerminal,
}

impl TuiSession {
    pub(crate) fn enter() -> Result<Self, String> {
        let terminal =
            ratatui::try_init().map_err(|e| format!("failed to start interactive mode: {e}"))?;
        Ok(Self { terminal })
    }
}

impl Drop for TuiSession {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
