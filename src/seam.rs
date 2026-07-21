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
//! - **SEAM-3 — mixed branch forms.** A norm that writes both the binary form
//!   (`antecedent` / `commitment` / `otherwise`) and the n-ary `cases:` form.
//!   The path to its commitment is then undefined, in three different ways:
//!   a stray `antecedent` beside `cases:` is read by nothing, so it is dead
//!   text that still passes LEAK and GROUND (an author would reasonably read it
//!   as a guard — a meaning §3 has not defined); `otherwise` beside a
//!   `when`-less case gives the norm two mutually exclusive residuals; and a
//!   top-level `commitment` beside `cases:` reads as unconditional, which
//!   contradicts splitting at all.
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

        // The two branch forms are alternatives, not layers: a norm written in
        // both has no well-defined path to its commitment (DESIGN §3).
        if norm.get("cases").is_some() {
            let mixed: Vec<&str> = ["antecedent", "commitment", "otherwise"]
                .into_iter()
                .filter(|k| norm.get(*k).is_some())
                .collect();
            if !mixed.is_empty() {
                out.push(Finding::new(
                    file,
                    &path,
                    Rule::MixedBranchForms,
                    format!(
                        "norm carries `cases:` alongside the binary form ({}) — the two are \
                         alternatives, not layers, so the path to its commitment is \
                         undefined; write one form or the other (DESIGN §3)",
                        mixed
                            .iter()
                            .map(|k| format!("`{k}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }
        }

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
