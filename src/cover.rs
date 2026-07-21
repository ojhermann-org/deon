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
//! The state space is coverage's input, so it is held to the same standard as
//! anything else this checker trusts (issue #18). [`bundle`] validates the
//! declarations themselves — once per bundle, not once per norm file that reads
//! it: **COVER-3** for a declaration that names no state at all (which would
//! otherwise vanish from the space silently, quietly weakening every COVER-1),
//! **COVER-4** for a block that looks like a declaration and yields nothing
//! (unparseable frontmatter, or a subject with no `states:` list), and
//! GROUND-1/2/3 for a declaration's citation, since "which states this subject
//! has" is a judgment about the standard and deon's rule for a judgment is that
//! it must cite where it grounds.
//!
//! COVER-3 and COVER-4 are the same failure at two altitudes: something that
//! reads as a state space silently yields nothing, and a state absent from the
//! space is one coverage stops looking for. Neither is visible in the norm
//! files that trust it.
//!
//! **Coverage is opt-in per norm.** A norm that claims no state makes no
//! coverage claim, and is skipped: nothing can be said about whether its
//! branches are total. Once a norm claims *one* state it must claim them all.
//! This is what lets a norm that is deliberately partial coexist with the
//! check, and it is why flagging the rev-rec seed's known gap required tagging
//! that seed's branches — the gap is reported because the norm asks to be
//! checked, not because the checker guessed.

use serde_yaml::Value;

use crate::{cases, ground, str_field, Finding, Okf, Rule};

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
            let Some(state) = state else {
                out.push(Finding::new(
                    file,
                    &format!("{path}.{branch}"),
                    Rule::UndeclaredStateClaimed,
                    "`covers:` names no state — it must be a state id, or a list of \
                     them; as written the branch claims nothing"
                        .to_string(),
                ));
                continue;
            };
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
            if !claims
                .iter()
                .any(|(_, claimed)| claimed.as_deref() == Some(state.as_str()))
            {
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
/// `otherwise`, or on each `cases[i]`. `None` marks a `covers:` that is present
/// but names no state — recognized so it can be reported, never skipped.
///
/// A branch may cover **several** states (`covers: [a, b]`): one branch of a
/// norm can legitimately handle more than one of the subject's states. That
/// shape is accepted rather than flagged — but it is *recognized* either way,
/// because a `covers:` the checker does not understand must not read as "this
/// norm makes no coverage claim". Silently opting a norm out of coverage is the
/// gap coverage exists to find.
fn claims(norm: &Value) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut claimed = |path: String, v: Option<&Value>| match v {
        None => {}
        Some(Value::String(s)) => out.push((path, Some(s.clone()))),
        Some(Value::Sequence(list)) => {
            for (i, item) in list.iter().enumerate() {
                let path = format!("{path}[{i}]");
                match item {
                    Value::String(s) => out.push((path, Some(s.clone()))),
                    _ => out.push((path, None)),
                }
            }
        }
        Some(_) => out.push((path, None)),
    };

    claimed("covers".to_string(), norm.get("covers"));
    if let Some(otherwise) = norm.get("otherwise") {
        claimed("otherwise.covers".to_string(), otherwise.get("covers"));
    }
    if let Some(Value::Sequence(list)) = norm.get("cases") {
        for ((case, _), value) in cases(norm).into_iter().zip(list) {
            claimed(format!("{case}.covers"), value.get("covers"));
        }
    }
    out
}

/// Validate the bundle's own state declarations (issue #18). Run once per
/// bundle by [`crate::check_bundle`] — a bundle is loaded once and checked
/// against many norm files, so reporting per file would duplicate every
/// finding. Findings are located in the *concept* file that declares them.
pub(crate) fn bundle(okf: &Okf, out: &mut Vec<Finding>) {
    for defect in okf.defects() {
        out.push(Finding::new(
            &defect.file,
            &defect.path,
            Rule::UnreadableStateSpace,
            defect.detail.clone(),
        ));
    }
    for decl in okf.declarations() {
        let path = format!("subjects.{}.states[{}]", decl.subject, decl.index);
        let Some(id) = &decl.id else {
            out.push(Finding::new(
                &decl.file,
                &path,
                Rule::MalformedState,
                format!(
                    "state declaration for subject `{}` names no state — it needs an `id` \
                     (or to be written as a bare string); as written it is dropped from the \
                     state space, so coverage silently stops checking for it",
                    decl.subject
                ),
            ));
            continue;
        };
        ground::criterion(
            &decl.file,
            &path,
            &decl.reference,
            &decl.source,
            okf,
            out,
            &format!("state `{id}`"),
        );
    }
}
