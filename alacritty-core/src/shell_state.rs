//! The shell state of a session, from the OSC 133 marks that the shell sends.
//!
//! The marks map to three states: `idle` after the prompt start (`A`) or the prompt end (`B`), `running` after the command start (`C`),
//! and `exited` after the command finished (`D`). [`ShellStateTracker::apply`] returns a [`Transition`] only when the state changes, so a
//! repeated mark emits nothing. The state is the sidecar's reading of the shell marks; a program that sends no marks leaves it unknown.

use crate::protocol::ShellMarker;

/// The shell state of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellState {
    Idle,
    Running,
    Exited,
}

impl ShellState {
    /// The name of the state in the events of the sidecar.
    pub fn name(self) -> &'static str {
        match self {
            ShellState::Idle => "idle",
            ShellState::Running => "running",
            ShellState::Exited => "exited",
        }
    }
}

/// A change of the shell state. `from` is None while no mark has been applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub from: Option<ShellState>,
    pub to: ShellState,
    /// One more than the sequence of the transition before it, starting at 1.
    pub sequence: u64,
}

/// Tracks the shell state of the session of one surface. A session that differs from the tracked one starts a new state.
#[derive(Debug, Default)]
pub struct ShellStateTracker {
    session: Option<String>,
    state: Option<ShellState>,
    sequence: u64,
}

impl ShellStateTracker {
    /// Applies one mark of the session and returns the transition that it causes, if any.
    pub fn apply(&mut self, session: &str, marker: ShellMarker) -> Option<Transition> {
        if self.session.as_deref() != Some(session) {
            self.session = Some(session.to_string());
            self.state = None;
            self.sequence = 0;
        }
        let to = match marker {
            ShellMarker::PromptStart | ShellMarker::PromptEnd => ShellState::Idle,
            ShellMarker::CommandStart => ShellState::Running,
            ShellMarker::CommandFinished => ShellState::Exited,
        };
        if self.state == Some(to) {
            return None;
        }
        let from = self.state.replace(to);
        self.sequence += 1;
        Some(Transition { from, to, sequence: self.sequence })
    }
}
