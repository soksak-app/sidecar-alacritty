// The supervisor classifies a dialog from declared patterns and never selects an option for a destructive or
// unclear dialog. A decision names its class and the pattern that decided it.
use soksak_sidecar_alacritty_core::supervise::{
    CancelReason, Decision, DialogClass, DialogOption, DialogText, decide,
};

fn dialog(text: &str) -> DialogText {
    DialogText {
        text: text.to_string(),
    }
}

fn options(labels: &[&str]) -> Vec<DialogOption> {
    labels
        .iter()
        .map(|label| DialogOption {
            label: label.to_string(),
        })
        .collect()
}

#[test]
fn destructive_dialog_cancels_and_names_its_pattern() {
    let decision = decide(
        &dialog("Delete 14 files in src/ with rm -rf?"),
        &options(&["Yes", "No"]),
    );
    assert_eq!(
        decision,
        Decision::Cancel {
            reason: CancelReason::Destructive,
            verdict: soksak_sidecar_alacritty_core::supervise::Verdict {
                class: DialogClass::Destructive,
                pattern: Some("destructive.delete"),
            },
        }
    );
}

#[test]
fn safe_dialog_selects_the_accepting_option_and_names_its_pattern() {
    let decision = decide(
        &dialog("Run the tests in alacritty-core?"),
        &options(&["No", "Yes, and allow cargo test for this session"]),
    );
    assert_eq!(
        decision,
        Decision::Answer {
            option: 1,
            verdict: soksak_sidecar_alacritty_core::supervise::Verdict {
                class: DialogClass::NonDestructive,
                pattern: Some("safe.run-tests"),
            },
        }
    );
}

#[test]
fn unclear_dialog_cancels_without_pattern() {
    let decision = decide(
        &dialog("Apply the pending change to the workspace?"),
        &options(&["Yes", "No"]),
    );
    assert_eq!(
        decision,
        Decision::Cancel {
            reason: CancelReason::Unclear,
            verdict: soksak_sidecar_alacritty_core::supervise::Verdict {
                class: DialogClass::Unclear,
                pattern: None,
            },
        }
    );
}

#[test]
fn destructive_option_label_makes_a_safe_dialog_cancel() {
    // The dialog text is safe, but the accepting option names a force push, so the dialog is destructive.
    let decision = decide(
        &dialog("Run the tests before pushing?"),
        &options(&["Yes, and force push afterwards", "No"]),
    );
    assert!(matches!(
        decision,
        Decision::Cancel {
            reason: CancelReason::Destructive,
            ..
        }
    ));
}

#[test]
fn safe_dialog_without_accepting_option_cancels() {
    let decision = decide(&dialog("Run the tests?"), &options(&["No", "Not now"]));
    assert!(matches!(
        decision,
        Decision::Cancel {
            reason: CancelReason::NoAcceptingOption,
            ..
        }
    ));
}
