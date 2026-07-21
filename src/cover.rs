//! Check 3 — coverage (DESIGN §4, check 3).
//!
//! The antecedent branches must partition the subject's relevant states. Spike
//! 1 found the motivating gap: IFRS 15's performance obligation has a *third*
//! state — "not yet satisfied → recognize nothing" — that an
//! over-time/point-in-time split leaves unrepresented (F5). An `otherwise` makes
//! that split _syntactically_ total without closing the real gap.
//!
//! The checker cannot know a subject's states; deriving them would be either an
//! evaluator (DESIGN §9) or a hardcoded accounting fact (out of scope — deon
//! owns the language, not the norms). So coverage does what every other check
//! here does: it makes the author **write the states down, cited, and verifies
//! the writing is discharged.** The state space is norm content, so it lives in
//! the OKF bundle beside its prose, and coverage — like GROUND-3 — runs only
//! under `--okf`.
//!
//! - **COVER-1 — uncovered state.** A state the subject declares that no branch
//!   of the norm claims. This is F5: the finding *names* the missing state
//!   rather than asking the author to prove a split is total.
//! - **COVER-2 — undeclared state claimed.** A `covers:` naming a state the
//!   subject does not declare (or claiming any state when the subject has no
//!   declared state space at all) — the dangling-anchor analogue.
//!
//! **Coverage is opt-in per norm.** A norm that claims no state makes no
//! coverage claim, and is skipped: nothing can be said about whether its
//! branches are total. Once a norm claims *one* state it must claim them all.
//! This is what lets a norm that is deliberately partial coexist with the
//! check, and it is why flagging the rev-rec seed's known gap required tagging
//! that seed's branches — the gap is reported because the norm asks to be
//! checked, not because the checker guessed.

use serde_yaml::Value;

use crate::{cases, str_field, Finding, Okf, Rule};

/// Run check 3 over the parsed document, appending findings.
pub(crate) fn check(doc: &Value, file: &str, okf: &Okf, out: &mut Vec<Finding>) {
    let Some(Value::Sequence(norms)) = doc.get("norms") else {
        return;
    };
    for (i, norm) in norms.iter().enumerate() {
        let Value::Mapping(_) = norm else { continue };
        let Some(subject) = str_field(norm, "subject") else {
            continue;
        };
        let path = format!("norms[{i}]");
        let claims = claims(norm);
        if claims.is_empty() {
            continue; // makes no coverage claim — nothing to check against
        }

        let declared = okf.states(&subject);
        for (branch, state) in &claims {
            if !declared.is_some_and(|d| d.contains(state)) {
                out.push(Finding::new(
                    file,
                    &format!("{path}.{branch}"),
                    Rule::UndeclaredStateClaimed,
                    match declared {
                        Some(_) => format!(
                            "`covers: {state}` names no state declared for subject \
                             `{subject}` in the OKF bundle"
                        ),
                        None => format!(
                            "`covers: {state}` claims a state, but the OKF bundle declares \
                             no state space for subject `{subject}`"
                        ),
                    },
                ));
            }
        }

        let Some(declared) = declared else { continue };
        for state in declared {
            if !claims.iter().any(|(_, claimed)| claimed == state) {
                out.push(Finding::new(
                    file,
                    &path,
                    Rule::UncoveredState,
                    format!(
                        "subject `{subject}` declares the state `{state}`, which no branch \
                         of this norm covers — an implicit gap the branch structure hides \
                         (DESIGN §4.3)"
                    ),
                ));
            }
        }
    }
}

/// Every state this norm claims, as `(branch path, state)`. A branch claims a
/// state with `covers:`: on the norm itself (the antecedent-holds branch), on
/// `otherwise`, or on each `cases[i]`.
fn claims(norm: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(state) = str_field(norm, "covers") {
        out.push(("covers".to_string(), state));
    }
    if let Some(otherwise) = norm.get("otherwise") {
        if let Some(state) = str_field(otherwise, "covers") {
            out.push(("otherwise.covers".to_string(), state));
        }
    }
    if let Some(Value::Sequence(list)) = norm.get("cases") {
        for ((case, _), value) in cases(norm).into_iter().zip(list) {
            if let Some(state) = str_field(value, "covers") {
                out.push((format!("{case}.covers"), state));
            }
        }
    }
    out
}
