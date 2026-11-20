// The shell state tracker: each OSC 133 mark maps to one state, and only a change of the state is a transition.
use soksak_sidecar_alacritty_core::protocol::ShellMarker;
use soksak_sidecar_alacritty_core::shell_state::{
    Current, ShellState, ShellStateTracker, Transition, exit_code,
};

fn params(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn each_change_of_the_state_is_one_transition_numbered_from_one() {
    let mut tracker = ShellStateTracker::default();
    assert_eq!(
        tracker.apply("s1", ShellMarker::PromptStart, &[]),
        Ok(Some(Transition {
            from: None,
            to: ShellState::Idle,
            sequence: 1,
            exit: None
        }))
    );
    assert_eq!(tracker.apply("s1", ShellMarker::PromptEnd, &[]), Ok(None));
    assert_eq!(
        tracker.apply("s1", ShellMarker::CommandStart, &[]),
        Ok(Some(Transition {
            from: Some(ShellState::Idle),
            to: ShellState::Running,
            sequence: 2,
            exit: None
        }))
    );
    assert_eq!(
        tracker.apply("s1", ShellMarker::CommandFinished, &params(&["3"])),
        Ok(Some(Transition {
            from: Some(ShellState::Running),
            to: ShellState::Exited,
            sequence: 3,
            exit: Some(3)
        }))
    );
    assert_eq!(
        tracker.current("s1"),
        Current {
            state: Some(ShellState::Exited),
            sequence: 3,
            exit: Some(3)
        }
    );
    assert_eq!(
        tracker.apply("s1", ShellMarker::PromptStart, &[]),
        Ok(Some(Transition {
            from: Some(ShellState::Exited),
            to: ShellState::Idle,
            sequence: 4,
            exit: Some(3)
        }))
    );
}

#[test]
fn a_session_that_differs_from_the_tracked_one_starts_a_new_state() {
    let mut tracker = ShellStateTracker::default();
    tracker.apply("s1", ShellMarker::CommandStart, &[]).unwrap();
    assert_eq!(
        tracker.current("s2"),
        Current {
            state: None,
            sequence: 0,
            exit: None
        }
    );
    assert_eq!(
        tracker.apply("s2", ShellMarker::PromptStart, &[]),
        Ok(Some(Transition {
            from: None,
            to: ShellState::Idle,
            sequence: 1,
            exit: None
        }))
    );
}

#[test]
fn the_exit_code_is_the_first_parameter_from_0_to_255() {
    assert_eq!(exit_code(&[]), Ok(None));
    assert_eq!(exit_code(&params(&["0"])), Ok(Some(0)));
    assert_eq!(exit_code(&params(&["255", "aid=1"])), Ok(Some(255)));
    assert_eq!(exit_code(&params(&["aid=1"])), Ok(None));
    assert!(exit_code(&params(&["256"])).is_err());
    assert!(exit_code(&params(&["-1"])).is_err());
    assert!(exit_code(&params(&["x"])).is_err());
    assert!(exit_code(&params(&[""])).is_err());
}

#[test]
fn a_malformed_exit_code_changes_no_state() {
    let mut tracker = ShellStateTracker::default();
    tracker.apply("s1", ShellMarker::CommandStart, &[]).unwrap();
    assert!(
        tracker
            .apply("s1", ShellMarker::CommandFinished, &params(&["x"]))
            .is_err()
    );
    assert_eq!(
        tracker.current("s1"),
        Current {
            state: Some(ShellState::Running),
            sequence: 1,
            exit: None
        }
    );
}

#[test]
fn the_state_names_are_the_event_names() {
    assert_eq!(ShellState::Idle.name(), "idle");
    assert_eq!(ShellState::Running.name(), "running");
    assert_eq!(ShellState::Exited.name(), "exited");
}
