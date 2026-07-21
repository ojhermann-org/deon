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
//!   references a bare name which is neither a subject field (`subject.field`)
//!   nor a declared, colored input — data of unknown provenance.
//! - **LEAK-3 — faked aggregation.** A `judgment-aggregation` node that also
//!   carries a `test`/`formula` — a weighed judgment faked as mechanical.

use std::collections::BTreeSet;

use serde_yaml::{Mapping, Value};

use crate::expr::tokenize;
use crate::{aggregation, is_judgment_color, key_str, Finding, Rule};

/// Words in a test expression that are not data references: the reserved
/// `threshold` artifact and boolean connectives.
const RESERVED: &[&str] = &["threshold", "and", "or", "not", "true", "false"];

/// Run check 1 over the parsed document, appending findings.
pub(crate) fn check(doc: &Value, file: &str, out: &mut Vec<Finding>) {
    let mut judgment = BTreeSet::new();
    collect_judgment_names(doc, &mut judgment);
    walk(doc, file, String::new(), &judgment, out);
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

/// Recursively walk, emitting a finding at each offending node.
fn walk(v: &Value, file: &str, path: String, judgment: &BTreeSet<String>, out: &mut Vec<Finding>) {
    match v {
        Value::Mapping(m) => {
            if let Some((test, inputs)) = mechanical_test(m) {
                scan_test(test, &inputs, judgment, file, &path, out);
            }
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
                walk(child, file, child_path, judgment, out);
            }
        }
        Value::Sequence(s) => {
            for (i, item) in s.iter().enumerate() {
                walk(item, file, format!("{path}[{i}]"), judgment, out);
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

/// Scan a mechanical `test` expression, emitting LEAK-1 / LEAK-2 as appropriate.
fn scan_test(
    test: &str,
    inputs: &BTreeSet<String>,
    judgment: &BTreeSet<String>,
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
        if tok.contains('.') {
            continue; // `subject.field` — a structured record access, seam data
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
