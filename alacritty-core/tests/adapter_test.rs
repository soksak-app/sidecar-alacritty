// The key adapter maps an action to the TUI's own key only when the declared table has one, and to the sok
// fallback otherwise. The table declares no key, so every action of every agent takes its fallback.
use soksak_sidecar_alacritty_core::adapter::{Action, Agent, KeyPlan, SokAction, key_for};

const AGENTS: [Agent; 2] = [Agent::Claude, Agent::Codex];
const ACTIONS: [Action; 5] = [
    Action::Cancel,
    Action::Confirm,
    Action::ProcessToBackground,
    Action::SendNow,
    Action::Interrupt,
];

#[test]
fn unknown_tui_action_gets_sok_fallback() {
    // Codex declares no send-now key, so the queued message is delivered at the next idle state.
    assert_eq!(
        key_for(Agent::Codex, Action::SendNow),
        KeyPlan::Fallback(SokAction::DeliverAtIdle)
    );
}

#[test]
fn cancel_without_tui_key_falls_back_to_interrupt() {
    assert_eq!(
        key_for(Agent::Claude, Action::Cancel),
        KeyPlan::Fallback(SokAction::Interrupt)
    );
}

#[test]
fn confirm_without_tui_key_has_no_sok_action() {
    assert_eq!(
        key_for(Agent::Claude, Action::Confirm),
        KeyPlan::Fallback(SokAction::Unavailable)
    );
}

#[test]
fn adapter_returns_no_key_for_actions_the_table_does_not_declare() {
    for agent in AGENTS {
        for action in ACTIONS {
            assert!(
                matches!(key_for(agent, action), KeyPlan::Fallback(_)),
                "{agent:?} {action:?} returned a TUI key that the table does not declare"
            );
        }
    }
}
