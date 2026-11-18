// The shell state tracker: each OSC 133 mark maps to one state, and only a change of the state is a transition.
use soksak_sidecar_alacritty_core::protocol::ShellMarker;
use soksak_sidecar_alacritty_core::shell_state::{ShellState, ShellStateTracker, Transition};

#[test]
fn each_change_of_the_state_is_one_transition_numbered_from_one() {
    let mut tracker = ShellStateTracker::default();
    assert_eq!(
        tracker.apply("s1", ShellMarker::PromptStart),
        Some(Transition { from: None, to: ShellState::Idle, sequence: 1 })
    );
    assert_eq!(tracker.apply("s1", ShellMarker::PromptEnd), None);
    assert_eq!(
        tracker.apply("s1", ShellMarker::CommandStart),
        Some(Transition { from: Some(ShellState::Idle), to: ShellState::Running, sequence: 2 })
    );
    assert_eq!(
        tracker.apply("s1", ShellMarker::CommandFinished),
        Some(Transition { from: Some(ShellState::Running), to: ShellState::Exited, sequence: 3 })
    );
    assert_eq!(
        tracker.apply("s1", ShellMarker::PromptStart),
        Some(Transition { from: Some(ShellState::Exited), to: ShellState::Idle, sequence: 4 })
    );
}

#[test]
fn a_session_that_differs_from_the_tracked_one_starts_a_new_state() {
    let mut tracker = ShellStateTracker::default();
    tracker.apply("s1", ShellMarker::CommandStart);
    assert_eq!(
        tracker.apply("s2", ShellMarker::PromptStart),
        Some(Transition { from: None, to: ShellState::Idle, sequence: 1 })
    );
}

#[test]
fn the_state_names_are_the_event_names() {
    assert_eq!(ShellState::Idle.name(), "idle");
    assert_eq!(ShellState::Running.name(), "running");
    assert_eq!(ShellState::Exited.name(), "exited");
}
