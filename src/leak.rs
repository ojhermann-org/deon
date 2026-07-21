//! Check 1 — leak detection (DESIGN §4, check 1): the mechanical edge.
//!
//! A *leak* is the mechanical/judgment seam being crossed **silently** — the
//! machine computing on something that is actually a judgment:
//!
//! - **LEAK-1 — judgment computed.** A `mechanical` test whose expression
//!   references a name declared `judgment`/`election` anywhere, *unless* that
//!   name is a declared **opaque input** of this test (an estimate crossing the
//!   seam as a value is fine; computing *on* a judgment is the leak).
//! - **LEAK-2 — undeclared / uncolored input.** A `mechanical` test that
//!   references a name which is neither a field of the norm's own `subject`
//!   (`subject.field`) nor a declared, colored input — data of unknown
//!   provenance. This covers **dotted** names too: a `record.field` whose root
//!   is not the subject is undeclared data, not seam data, and an input
//!   declared with no color (or a color that is not one of the three) is not a
//!   declaration.
//! - **LEAK-3 — faked aggregation.** A `judgment-aggregation` node that also
//!   carries a `test`/`formula` — a weighed judgment faked as mechanical.
//! - **LEAK-4 — uncolored commitment field.** A commitment's `method`/`measure`
//!   written without a color. Whether a method is an open choice or is
//!   *prescribed* by the standard is itself standard-relative (spike 2, N1): the
//!   progress measure in IFRS 15.39 is a judgment, while the retrospective
//!   restatement IAS 8 requires for a material error is determined. The checker
//!   cannot tell which from the value, so the norm must say — exactly as a
//!   threshold does (`{ value: 0.75, regime: ASC-840, color: mechanical }`).
//!
//! **The boundary, stated plainly.** These rules verify that an author was
//! *internally consistent* about the seam — that a name declared judgment is not
//! then computed on, and that computed-on data was declared. They cannot verify
//! that a colored `mechanical` predicate *deserves* that color: `lease.is-
//! specialized` is indistinguishable from any other boolean field of the
//! subject, and deciding otherwise would need the accounting knowledge this
//! checker deliberately does not hold. A judgment can still be laundered by
//! writing it as a field of its own subject. That is the honest limit of a
//! static check here, and the README says so rather than implying Lean-style
//! derivation.

use std::collections::BTreeSet;

use serde_yaml::{Mapping, Value};

use crate::expr::tokenize;
use crate::{aggregation, is_judgment_color, key_str, str_field, Finding, Rule};

/// Words in a test expression that are not data references: the reserved
/// `threshold` artifact and boolean connectives.
const RESERVED: &[&str] = &["threshold", "and", "or", "not", "true", "false"];

/// Run check 1 over the parsed document, appending findings.
pub(crate) fn check(doc: &Value, file: &str, out: &mut Vec<Finding>) {
    let mut judgment = BTreeSet::new();
    collect_judgment_names(doc, &mut judgment);
    walk(doc, file, String::new(), &judgment, None, out);
}

/// Gather every name declared with `color: judgment|election` — both
/// `predicate:` declarations and `inputs:` entries — anywhere in the tree.
fn collect_judgment_names(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Mapping(m) => {
            if let (Some(Value::String(name)), Some(Value::String(color))) =
                (m.get("predicate"), m.get("color"))
            {
                if is_judgment_color(color) {
                    out.insert(name.clone());
                }
            }
            for (name, color) in declared_inputs(m) {
                if color.as_deref().is_some_and(is_judgment_color) {
                    out.insert(name);
                }
            }
            for (_k, child) in m {
                collect_judgment_names(child, out);
            }
        }
        Value::Sequence(s) => {
            for item in s {
                collect_judgment_names(item, out);
            }
        }
        _ => {}
    }
}

/// Recursively walk, emitting a finding at each offending node. `subject`
/// carries the enclosing norm's `subject:` down the tree — a dotted name is only
/// seam data if it is rooted in the record the norm ranges over.
fn walk(
    v: &Value,
    file: &str,
    path: String,
    judgment: &BTreeSet<String>,
    subject: Option<&str>,
    out: &mut Vec<Finding>,
) {
    match v {
        Value::Mapping(m) => {
            let subject = match m.get("subject") {
                Some(Value::String(s)) => Some(s.as_str()),
                _ => subject,
            };
            if let Some((test, inputs)) = mechanical_test(m) {
                scan_test(test, &inputs, judgment, subject, file, &path, out);
                scan_input_colors(m, file, &path, out);
            }
            scan_commitment_colors(m, file, &path, out);
            if let Some(agg) = aggregation(m) {
                if agg.contains_key("test") || agg.contains_key("formula") {
                    out.push(Finding::new(
                        file,
                        &path,
                        Rule::FakedAggregation,
                        "judgment-aggregation carries a formula/test — a weighed judgment \
                         faked as a mechanical combination rule"
                            .to_string(),
                    ));
                }
            }
            for (k, child) in m {
                let seg = key_str(k);
                let child_path = if path.is_empty() {
                    seg
                } else {
                    format!("{path}.{seg}")
                };
                walk(child, file, child_path, judgment, subject, out);
            }
        }
        Value::Sequence(s) => {
            for (i, item) in s.iter().enumerate() {
                walk(item, file, format!("{path}[{i}]"), judgment, subject, out);
            }
        }
        _ => {}
    }
}

/// If this mapping is a mechanical test, return its `test` expression and the
/// set of names declared as its inputs. Handles both concrete shapes:
///   - inline:  `{ predicate: _, color: mechanical, test: <expr>, inputs: {..} }`
///   - nested:  `{ mechanical: { test: <expr>, inputs: {..} } }`
fn mechanical_test(m: &Mapping) -> Option<(&str, BTreeSet<String>)> {
    if let Some(Value::Mapping(inner)) = m.get("mechanical") {
        if let Some(Value::String(test)) = inner.get("test") {
            return Some((test, input_names(inner)));
        }
    }
    if m.get("color") == Some(&Value::String("mechanical".into())) {
        if let Some(Value::String(test)) = m.get("test") {
            return Some((test, input_names(m)));
        }
    }
    None
}

/// Names declared under this mapping's `inputs:` (keys of the inputs mapping).
fn input_names(m: &Mapping) -> BTreeSet<String> {
    declared_inputs(m)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// `(name, color)` for each entry under `inputs:`; color is `None` if the entry
/// carries no `color` field.
fn declared_inputs(m: &Mapping) -> Vec<(String, Option<String>)> {
    let Some(Value::Mapping(inputs)) = m.get("inputs") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (k, spec) in inputs {
        let Value::String(name) = k else { continue };
        let color = match spec {
            Value::Mapping(sm) => match sm.get("color") {
                Some(Value::String(c)) => Some(c.clone()),
                _ => None,
            },
            _ => None,
        };
        out.push((name.clone(), color));
    }
    out
}

/// A commitment's `method`/`measure` must declare which side of the seam it
/// falls on. Reached through the generic walk, so a residual `otherwise` and
/// every `cases[i]` commitment are covered as well as the norm's own.
fn scan_commitment_colors(m: &Mapping, file: &str, path: &str, out: &mut Vec<Finding>) {
    let Some(Value::Mapping(commitment)) = m.get("commitment") else {
        return;
    };
    for field in ["method", "measure"] {
        let Some(value) = commitment.get(field) else {
            continue;
        };
        let color = match value {
            Value::Mapping(_) => str_field(value, "color"),
            _ => None,
        };
        let declared = color
            .as_deref()
            .is_some_and(|c| c == "mechanical" || is_judgment_color(c));
        if !declared {
            out.push(Finding::new(
                file,
                &format!("{path}.commitment.{field}"),
                Rule::UncoloredCommitmentField,
                match color {
                    Some(c) => format!(
                        "commitment `{field}` declares color `{c}` — must be one of \
                         mechanical | judgment | election"
                    ),
                    None => format!(
                        "commitment `{field}` declares no color — whether a method is an \
                         open choice or is prescribed by the standard is standard-relative \
                         (spike 2, N1), so the norm must say which"
                    ),
                },
            ));
        }
    }
}

/// Every declared input must carry one of the three colors. Listing a name
/// under `inputs:` is not a declaration if it says nothing about which side of
/// the seam the value comes from — and without this, adding an empty entry is
/// the one-line way to silence LEAK-2 for that name.
fn scan_input_colors(m: &Mapping, file: &str, path: &str, out: &mut Vec<Finding>) {
    let inner = match m.get("mechanical") {
        Some(Value::Mapping(inner)) => inner,
        _ => m,
    };
    for (name, color) in declared_inputs(inner) {
        let colored = color
            .as_deref()
            .is_some_and(|c| c == "mechanical" || is_judgment_color(c));
        if !colored {
            out.push(Finding::new(
                file,
                &format!("{path}.inputs.{name}"),
                Rule::UndeclaredInput,
                match color {
                    Some(c) => format!(
                        "input `{name}` is declared with color `{c}` — must be one of \
                         mechanical | judgment | election"
                    ),
                    None => format!(
                        "input `{name}` is declared with no color — listing a name under \
                         `inputs:` does not say which side of the seam its value comes from"
                    ),
                },
            ));
        }
    }
}

/// Scan a mechanical `test` expression, emitting LEAK-1 / LEAK-2 as appropriate.
fn scan_test(
    test: &str,
    inputs: &BTreeSet<String>,
    judgment: &BTreeSet<String>,
    subject: Option<&str>,
    file: &str,
    path: &str,
    out: &mut Vec<Finding>,
) {
    for (tok, is_call) in tokenize(test) {
        if is_call || RESERVED.contains(&tok.as_str()) {
            continue; // function application or reserved word — not a data ref
        }
        if tok.starts_with(|c: char| c.is_ascii_digit()) {
            continue; // numeric literal
        }
        // A dotted name is seam data only if it is rooted in the norm's own
        // subject or in a declared input. Waving every `a.b` through let a
        // judgment be laundered by appending a field access to its name.
        let root = tok.split('.').next().unwrap_or(&tok).to_string();
        if tok.contains('.') {
            if judgment.contains(&root) {
                out.push(Finding::new(
                    file,
                    path,
                    Rule::JudgmentComputed,
                    format!(
                        "mechanical test computes on `{tok}`, rooted in judgment/election \
                         name `{root}` — a field access does not make a judgment mechanical"
                    ),
                ));
            } else if !inputs.contains(&root) && Some(root.as_str()) != subject {
                out.push(Finding::new(
                    file,
                    path,
                    Rule::UndeclaredInput,
                    format!(
                        "mechanical test references `{tok}`, rooted in `{root}` which is \
                         neither this norm's subject nor a declared colored input"
                    ),
                ));
            }
            continue;
        }
        if inputs.contains(&tok) {
            continue; // declared opaque input — allowed to cross the seam as a value
        }
        if judgment.contains(&tok) {
            out.push(Finding::new(
                file,
                path,
                Rule::JudgmentComputed,
                format!(
                    "mechanical test computes on judgment/election name `{tok}`, \
                     not declared as an opaque input of this test"
                ),
            ));
        } else {
            out.push(Finding::new(
                file,
                path,
                Rule::UndeclaredInput,
                format!(
                    "mechanical test references `{tok}`, neither a subject field \
                     (`subject.field`) nor a declared colored input"
                ),
            ));
        }
    }
}
