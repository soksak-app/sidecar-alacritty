//! Maps the abstract actions of the terminal to the key that each agent TUI supports, or to the sok
//! fallback when the TUI has no declared key for the action. The caller names the action; it never
//! names the key.

use crate::protocol::InputKey;

/// The agent whose TUI receives the key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Codex,
}

/// An action that the terminal performs on an agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Dismisses a dialog without choosing an option.
    Cancel,
    /// Accepts the selected option of a dialog.
    Confirm,
    /// Sends the running foreground command to the background.
    ProcessToBackground,
    /// Delivers the input now, while the TUI is busy.
    SendNow,
    /// Interrupts the running turn.
    Interrupt,
}

/// The sok action that stands in for a TUI key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SokAction {
    /// Sends an interrupt to the foreground job of the terminal.
    Interrupt,
    /// Delivers a queued message at the next idle state of the terminal.
    DeliverAtIdle,
    /// No sok action performs this action. The caller logs the action as not delivered.
    Unavailable,
}

/// How an action reaches the agent: the TUI's own keys, or a sok action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyPlan {
    Tui(Vec<InputKey>),
    Fallback(SokAction),
}

/// One key of a TUI as declared data. `text` is empty for named keys and holds the character for `Char`.
struct TuiKey {
    key: &'static str,
    text: &'static str,
    shift: bool,
    alt: bool,
    ctrl: bool,
}

impl TuiKey {
    fn input(&self) -> InputKey {
        InputKey {
            key: self.key.to_string(),
            text: self.text.to_string(),
            shift: self.shift,
            alt: self.alt,
            ctrl: self.ctrl,
        }
    }
}

struct TuiKeyEntry {
    agent: Agent,
    action: Action,
    keys: &'static [TuiKey],
}

/// The keys that each agent TUI supports for each action.
///
/// The table declares a key only with the agent version that the key was observed on. It is empty,
/// so every action takes its fallback. A key that is not observed is never
/// listed here.
const TUI_KEYS: &[TuiKeyEntry] = &[];

/// The sok action for an action that the TUI has no declared key for.
fn fallback_for(action: Action) -> SokAction {
    match action {
        // The terminal's interrupt is the cancel path when the TUI has no cancel key.
        Action::Cancel => SokAction::Interrupt,
        // No sok action selects a dialog option or accepts a dialog.
        Action::Confirm => SokAction::Unavailable,
        // No sok action moves a foreground command to the background.
        Action::ProcessToBackground => SokAction::Unavailable,
        // A message that cannot be sent now is delivered at the next idle state.
        Action::SendNow => SokAction::DeliverAtIdle,
        Action::Interrupt => SokAction::Interrupt,
    }
}

/// Returns the TUI keys of the agent for the action when the table declares them, and the sok
/// fallback of the action otherwise.
pub fn key_for(agent: Agent, action: Action) -> KeyPlan {
    match TUI_KEYS
        .iter()
        .find(|entry| entry.agent == agent && entry.action == action)
    {
        Some(entry) => KeyPlan::Tui(entry.keys.iter().map(TuiKey::input).collect()),
        None => KeyPlan::Fallback(fallback_for(action)),
    }
}
