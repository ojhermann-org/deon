//! Check 5 — termination-at-seam (DESIGN §4, check 5).
//!
//! Every norm's obligation must reach a `commitment` about plain data — the
//! bottom edge where deon hands off to the Lean seam. A norm that constrains no
//! seam data is malformed (DESIGN §2). This is the most mechanical check: it
//! iterates the `norms` list and asserts each entry terminates.
//!
//! A norm reaches the seam through **either** a non-empty `commitment` **or**
//! (for a defeat entry) a non-empty `modifies` that changes the defeated norm's
//! commitment. Two failure modes:
//!
//! - **SEAM-1 — unreached seam.** A norm with neither a `commitment` nor a
//!   `modifies`: its obligation leads nowhere.
//! - **SEAM-2 — empty commitment.** A `commitment` / `modifies` that is present
//!   but empty, or an `otherwise` residual branch — or a `cases:` branch — that
//!   carries no commitment: a branch that constrains no plain data.
//!
//! Both branch forms of DESIGN §3 are checked: a norm written in the n-ary
//! `cases:` form reaches the seam through its cases, and each case is held to
//! the same standard as a residual `otherwise`.

use serde_yaml::Value;

use crate::{cases, Finding, Rule};

/// Run check 5 over the parsed document, appending findings.
pub(crate) fn check(doc: &Value, file: &str, out: &mut Vec<Finding>) {
    let Some(Value::Sequence(norms)) = doc.get("norms") else {
        return;
    };
    for (i, norm) in norms.iter().enumerate() {
        let path = format!("norms[{i}]");
        let Value::Mapping(_) = norm else { continue };

        let commitment = norm.get("commitment");
        let modifies = norm.get("modifies");
        let cases = cases(norm);

        if commitment.is_none() && modifies.is_none() && cases.is_empty() {
            out.push(Finding::new(
                file,
                &path,
                Rule::UnreachedSeam,
                "norm reaches no commitment about plain data — it has neither a \
                 `commitment` nor a `modifies`, so its obligation leads nowhere"
                    .to_string(),
            ));
        } else {
            if commitment.is_some() && !is_nonempty(commitment) {
                out.push(Finding::new(
                    file,
                    &format!("{path}.commitment"),
                    Rule::EmptyCommitment,
                    "commitment is empty — it constrains no plain data".to_string(),
                ));
            }
            if modifies.is_some() && !is_nonempty(modifies) {
                out.push(Finding::new(
                    file,
                    &format!("{path}.modifies"),
                    Rule::EmptyCommitment,
                    "modifies is empty — it changes no commitment".to_string(),
                ));
            }
        }

        // Every `cases:` branch must carry a commitment of its own — a case is a
        // branch of the norm, so a case that commits nothing leads nowhere just
        // as a dead-end `otherwise` does.
        for (case, commitment) in &cases {
            if !is_nonempty(*commitment) {
                out.push(Finding::new(
                    file,
                    &format!("{path}.{case}"),
                    Rule::EmptyCommitment,
                    "case carries no commitment about plain data — this branch of the \
                     norm leads nowhere"
                        .to_string(),
                ));
            }
        }

        // A residual `otherwise` branch must itself carry a commitment, or that
        // branch of the norm leads nowhere.
        if let Some(otherwise) = norm.get("otherwise") {
            if !is_nonempty(otherwise.get("commitment")) {
                out.push(Finding::new(
                    file,
                    &format!("{path}.otherwise"),
                    Rule::EmptyCommitment,
                    "residual `otherwise` branch carries no commitment about plain data"
                        .to_string(),
                ));
            }
        }
    }
}

/// A value that actually constrains something: a non-empty mapping.
fn is_nonempty(v: Option<&Value>) -> bool {
    matches!(v, Some(Value::Mapping(m)) if !m.is_empty())
}
