//! Check 2 — grounding completeness (DESIGN §4, check 2): the judgment edge.
//!
//! Every judgment *hole* must carry a citation, so no open-textured predicate
//! floats ungrounded. The load-bearing distinction (settled in issue #10) is
//! **concept vs. value**:
//!
//! - A **criterion** — a judgment/election predicate, a judgment `threshold`, a
//!   `judgment-aggregation`, a judgment `binds`, a judgment commitment field
//!   (`method`/`measure`), a `violated(status: judgment)` — names an
//!   open-textured test. It resolves to *concept prose*, so it needs a
//!   `grounds.ref` (+ a typed `source`).
//! - A judgment-colored **input** to a mechanical test is a *value* (an
//!   estimate). It resolves to *runtime evidence*, not norm-time prose, so a
//!   `ref` would be a category error: it is grounded by its `source` type alone.
//!
//! Rules:
//! - **GROUND-1 — missing citation.** A criterion with no `grounds.ref`.
//! - **GROUND-2 — invalid source.** A citation (criterion or input) whose
//!   `source` is absent or not one of the four declared types.
//! - **GROUND-3 — dangling anchor.** A well-formed criterion `ref` that does not
//!   resolve in the OKF bundle (only under `--okf`; see [`crate::Okf`]).

use serde_yaml::Value;

use crate::{aggregation, is_judgment_color, key_str, str_field, Finding, Okf, Rule, SOURCE_TYPES};

/// What kind of grounding a node needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A criterion: needs a `ref` (and a valid `source`).
    Criterion,
    /// A judgment-colored input: needs only a valid `source`.
    Input,
}

/// A judgment node carrying (or lacking) a citation.
struct Node {
    path: String,
    kind: Kind,
    reference: Option<String>,
    source: Option<String>,
}

/// GROUND-1 / GROUND-2: structural grounding, no OKF bundle needed.
pub(crate) fn structural(doc: &Value, file: &str, out: &mut Vec<Finding>) {
    for node in collect(doc) {
        match node.kind {
            Kind::Criterion if node.reference.is_none() => out.push(Finding::new(
                file,
                &node.path,
                Rule::MissingCitation,
                "judgment criterion carries no `grounds.ref` — an open-textured hole \
                 must cite where it grounds"
                    .to_string(),
            )),
            _ if !valid_source(&node.source) => out.push(Finding::new(
                file,
                &node.path,
                Rule::InvalidSource,
                format!(
                    "`grounds.source` is {} — must be one of {}",
                    match &node.source {
                        Some(s) => format!("`{s}`"),
                        None => "absent".to_string(),
                    },
                    SOURCE_TYPES.join(" | ")
                ),
            )),
            _ => {}
        }
    }
}

/// GROUND-3: every well-formed criterion `ref` must resolve in `okf`.
pub(crate) fn anchors(doc: &Value, file: &str, okf: &Okf, out: &mut Vec<Finding>) {
    for node in collect(doc) {
        // Only well-formed criteria (ref present + valid source) are eligible;
        // a node already flagged by GROUND-1/2 is not re-reported here.
        if node.kind != Kind::Criterion || !valid_source(&node.source) {
            continue;
        }
        if let Some(reference) = &node.reference {
            if !okf.resolves(reference) {
                out.push(Finding::new(
                    file,
                    &node.path,
                    Rule::DanglingAnchor,
                    format!("`grounds.ref` `{reference}` does not resolve to an OKF anchor"),
                ));
            }
        }
    }
}

fn valid_source(source: &Option<String>) -> bool {
    source.as_deref().is_some_and(|s| SOURCE_TYPES.contains(&s))
}

/// Walk the document, collecting every judgment node (criteria and inputs).
fn collect(doc: &Value) -> Vec<Node> {
    let mut out = Vec::new();
    walk(doc, String::new(), &mut out);
    out
}

fn walk(v: &Value, path: String, out: &mut Vec<Node>) {
    match v {
        Value::Mapping(m) => {
            let colored = str_field(v, "color").is_some_and(|c| is_judgment_color(&c));
            if colored || aggregation(m).is_some() {
                let grounds = m.get("grounds");
                out.push(Node {
                    path: path.clone(),
                    kind: Kind::Criterion,
                    reference: grounds.and_then(|g| str_field(g, "ref")),
                    source: grounds.and_then(|g| str_field(g, "source")),
                });
            }
            for (k, child) in m {
                let seg = key_str(k);
                let child_path = if path.is_empty() {
                    seg.clone()
                } else {
                    format!("{path}.{seg}")
                };
                // `inputs` entries are values, not criteria: check each for a
                // valid source, and do not descend (so they are never treated
                // as criteria that need a `ref`).
                if seg == "inputs" {
                    if let Value::Mapping(inputs) = child {
                        for (name, spec) in inputs {
                            if str_field(spec, "color").is_some_and(|c| is_judgment_color(&c)) {
                                out.push(Node {
                                    path: format!("{child_path}.{}", key_str(name)),
                                    kind: Kind::Input,
                                    reference: str_field(spec, "ref"),
                                    source: str_field(spec, "source"),
                                });
                            }
                        }
                    }
                } else {
                    walk(child, child_path, out);
                }
            }
        }
        Value::Sequence(s) => {
            for (i, item) in s.iter().enumerate() {
                walk(item, format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}
