//! Check 6 — regime hygiene (DESIGN §4, check 6).
//!
//! A norm applies only within its `regime`. Norms are regime-indexed (DESIGN
//! §3), inheriting the document's `regime:` unless they override it (as the
//! lease seed's `short-term-low-value-election` overrides to `IFRS-16`).
//!
//! The evaluation-time facet — flagging *facts* checked against a norm whose
//! regime doesn't apply — needs runtime facts and is deferred (DESIGN §9). The
//! representational facet, checkable now, is that a norm's own mechanized
//! artifacts must belong to its regime:
//!
//! - **REGIME-1 — undetermined regime.** A norm with no effective regime
//!   (neither its own nor an inherited document `regime:`): it cannot be scoped.
//! - **REGIME-2 — cross-regime artifact.** A `@regime`-stamped artifact (a
//!   `threshold`) inside a norm whose regime differs — a bright-line pulled from
//!   a regime this norm does not inhabit.
//!
//! Cross-regime `defeats` edges are **not** flagged here: the lease seed uses
//! one deliberately (an `IFRS-16` election defeating the `ASC-840` classification
//! to model regime-relativity, per its `regime-note`), so treating it as a
//! violation is a separate, contestable call — left for a future refinement.

use serde_yaml::Value;

use crate::{key_str, str_field, Finding, Rule};

/// Run check 6 over the parsed document, appending findings.
pub(crate) fn check(doc: &Value, file: &str, out: &mut Vec<Finding>) {
    let doc_regime = str_field(doc, "regime");
    let Some(Value::Sequence(norms)) = doc.get("norms") else {
        return;
    };
    for (i, norm) in norms.iter().enumerate() {
        let Value::Mapping(_) = norm else { continue };
        let path = format!("norms[{i}]");
        match str_field(norm, "regime").or_else(|| doc_regime.clone()) {
            None => out.push(Finding::new(
                file,
                &path,
                Rule::UndeterminedRegime,
                "norm has no regime and the document declares none — norms are \
                 regime-indexed (DESIGN §3)"
                    .to_string(),
            )),
            Some(effective) => walk(norm, &path, &effective, file, out),
        }
    }
}

/// Walk a norm's subtree; flag any `@regime`-stamped artifact whose regime is
/// not the norm's effective regime. (The norm's own `regime:` field equals
/// `effective` by construction, so it never self-flags.)
fn walk(v: &Value, path: &str, effective: &str, file: &str, out: &mut Vec<Finding>) {
    match v {
        Value::Mapping(m) => {
            if let Some(regime) = str_field(v, "regime") {
                if regime != effective {
                    out.push(Finding::new(
                        file,
                        path,
                        Rule::CrossRegimeArtifact,
                        format!(
                            "artifact stamped `@{regime}` in a norm scoped to `{effective}` — \
                             a bright-line from a regime this norm does not inhabit"
                        ),
                    ));
                }
            }
            for (k, child) in m {
                walk(
                    child,
                    &format!("{path}.{}", key_str(k)),
                    effective,
                    file,
                    out,
                );
            }
        }
        Value::Sequence(s) => {
            for (i, item) in s.iter().enumerate() {
                walk(item, &format!("{path}[{i}]"), effective, file, out);
            }
        }
        _ => {}
    }
}
