//! Decides how the supervisor answers a dialog that a terminal shows. A dialog is destructive when its
//! text or an option label matches a declared destructive pattern, and non-destructive when it matches
//! a declared safe pattern and an option accepts. Any other dialog is unclear. The supervisor cancels
//! a destructive or unclear dialog and never selects an option for it.

/// The class of a dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogClass {
    Destructive,
    NonDestructive,
    Unclear,
}

/// The text of the dialog that a terminal shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogText {
    pub text: String,
}

/// One option of the dialog, as its label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogOption {
    pub label: String,
}

/// Why the supervisor cancels a dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelReason {
    Destructive,
    Unclear,
    /// The dialog matched a safe pattern, but no option accepts.
    NoAcceptingOption,
}

/// The class of a decision and the id of the pattern that decided it. `pattern` is None when no
/// pattern matched, which is the unclear case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub class: DialogClass,
    pub pattern: Option<&'static str>,
}

/// The answer to a dialog: the index of the option to select, or the reason to cancel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Answer {
        option: usize,
        verdict: Verdict,
    },
    Cancel {
        reason: CancelReason,
        verdict: Verdict,
    },
}

/// A declared pattern: the text that matches it is compared in lowercase ASCII.
struct Pattern {
    id: &'static str,
    needle: &'static str,
}

/// Patterns of destructive dialogs. A dialog that matches one is cancelled, so the list errs toward
/// matching: a false match cancels a dialog, and a missed match would answer a destructive one.
const DESTRUCTIVE: &[Pattern] = &[
    Pattern {
        id: "destructive.delete",
        needle: "delete",
    },
    Pattern {
        id: "destructive.remove",
        needle: "remove",
    },
    Pattern {
        id: "destructive.rm",
        needle: "rm -",
    },
    Pattern {
        id: "destructive.overwrite",
        needle: "overwrit",
    },
    Pattern {
        id: "destructive.discard",
        needle: "discard",
    },
    Pattern {
        id: "destructive.uncommitted",
        needle: "reset --hard",
    },
    Pattern {
        id: "destructive.uncommitted",
        needle: "git clean",
    },
    Pattern {
        id: "destructive.uncommitted",
        needle: "checkout --",
    },
    Pattern {
        id: "destructive.history",
        needle: "rebase",
    },
    Pattern {
        id: "destructive.history",
        needle: "amend",
    },
    Pattern {
        id: "destructive.history",
        needle: "filter-branch",
    },
    Pattern {
        id: "destructive.history",
        needle: "filter-repo",
    },
    Pattern {
        id: "destructive.force-push",
        needle: "force",
    },
    Pattern {
        id: "destructive.credential",
        needle: "credential",
    },
    Pattern {
        id: "destructive.credential",
        needle: "password",
    },
    Pattern {
        id: "destructive.credential",
        needle: "token",
    },
    Pattern {
        id: "destructive.credential",
        needle: "secret",
    },
    Pattern {
        id: "destructive.permission",
        needle: "permission",
    },
    Pattern {
        id: "destructive.permission",
        needle: "chmod",
    },
    Pattern {
        id: "destructive.permission",
        needle: "chown",
    },
    Pattern {
        id: "destructive.permission",
        needle: "sudo",
    },
    Pattern {
        id: "destructive.process",
        needle: "kill",
    },
    Pattern {
        id: "destructive.process",
        needle: "terminate",
    },
    Pattern {
        id: "destructive.external-write",
        needle: "upload",
    },
    Pattern {
        id: "destructive.external-write",
        needle: "publish",
    },
    Pattern {
        id: "destructive.external-write",
        needle: "push",
    },
    Pattern {
        id: "destructive.external-write",
        needle: "send to",
    },
    Pattern {
        id: "destructive.external-write",
        needle: "post to",
    },
];

/// Patterns of dialogs that are reversible and scoped to the project. A dialog that matches one and
/// has an accepting option is answered.
const SAFE: &[Pattern] = &[
    Pattern {
        id: "safe.run-tests",
        needle: "run the tests",
    },
    Pattern {
        id: "safe.run-tests",
        needle: "npm test",
    },
    Pattern {
        id: "safe.run-tests",
        needle: "pnpm test",
    },
    Pattern {
        id: "safe.run-tests",
        needle: "cargo test",
    },
    Pattern {
        id: "safe.run-tests",
        needle: "go test",
    },
    Pattern {
        id: "safe.read-project-file",
        needle: "read the file",
    },
    Pattern {
        id: "safe.read-project-file",
        needle: "reading file",
    },
    Pattern {
        id: "safe.continue-paused-turn",
        needle: "paused turn",
    },
];

/// Label prefixes of an option that accepts a dialog.
const ACCEPT_PREFIXES: &[&str] = &["yes", "allow", "continue", "proceed", "resume", "run"];

fn first_match(patterns: &'static [Pattern], haystack: &str) -> Option<&'static str> {
    patterns
        .iter()
        .find(|pattern| haystack.contains(pattern.needle))
        .map(|pattern| pattern.id)
}

fn accepts(option: &DialogOption) -> bool {
    let label = option.label.trim().to_ascii_lowercase();
    ACCEPT_PREFIXES
        .iter()
        .any(|prefix| label.starts_with(prefix))
}

/// Decides the answer to a dialog. The dialog text and every option label are matched, so an option
/// that names a destructive action makes the dialog destructive.
pub fn decide(dialog: &DialogText, options: &[DialogOption]) -> Decision {
    let mut haystack = dialog.text.to_ascii_lowercase();
    for option in options {
        haystack.push('\n');
        haystack.push_str(&option.label.to_ascii_lowercase());
    }
    if let Some(pattern) = first_match(DESTRUCTIVE, &haystack) {
        return Decision::Cancel {
            reason: CancelReason::Destructive,
            verdict: Verdict {
                class: DialogClass::Destructive,
                pattern: Some(pattern),
            },
        };
    }
    if let Some(pattern) = first_match(SAFE, &haystack) {
        let verdict = Verdict {
            class: DialogClass::NonDestructive,
            pattern: Some(pattern),
        };
        return match options.iter().position(accepts) {
            Some(option) => Decision::Answer { option, verdict },
            None => Decision::Cancel {
                reason: CancelReason::NoAcceptingOption,
                verdict,
            },
        };
    }
    Decision::Cancel {
        reason: CancelReason::Unclear,
        verdict: Verdict {
            class: DialogClass::Unclear,
            pattern: None,
        },
    }
}
