//! deon leak-detection static check (DESIGN §4, check 1).
//!
//! A *leak* is the mechanical/judgment seam being crossed **silently** — the
//! machine computing on something that is actually a judgment. Over the
//! OKF-frontmatter norm schema this checker walks every node and flags three
//! shapes:
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
//!
//! This is representational-only: parse frontmatter, walk the normal form, flag.
//! Nothing is evaluated.

use std::collections::BTreeSet;
use std::fmt;

use serde_yaml::{Mapping, Value};

/// The three leak shapes (DESIGN §4, check 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leak {
    /// A mechanical test computes on a judgment/election name.
    JudgmentComputed,
    /// A mechanical test references an undeclared / uncolored input.
    UndeclaredInput,
    /// A judgment-aggregation carries a formula/test.
    FakedAggregation,
}

impl Leak {
    /// Stable short code, e.g. `LEAK-1`.
    pub fn code(self) -> &'static str {
        match self {
            Leak::JudgmentComputed => "LEAK-1",
            Leak::UndeclaredInput => "LEAK-2",
            Leak::FakedAggregation => "LEAK-3",
        }
    }

    /// Human-readable rule name.
    pub fn title(self) -> &'static str {
        match self {
            Leak::JudgmentComputed => "judgment computed",
            Leak::UndeclaredInput => "undeclared/uncolored input",
            Leak::FakedAggregation => "faked aggregation",
        }
    }
}

/// A located leak: which file, which node, which rule, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Source file the leak was found in.
    pub file: String,
    /// Node path into the frontmatter, e.g. `norms[0].antecedent.over-time`.
    pub path: String,
    /// Which rule tripped.
    pub leak: Leak,
    /// One-line explanation.
    pub detail: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}\t{} ({}): {}",
            self.file,
            self.path,
            self.leak.code(),
            self.leak.title(),
            self.detail
        )
    }
}

/// Words in a test expression that are not data references: the reserved
/// `threshold` artifact and boolean connectives.
const RESERVED: &[&str] = &["threshold", "and", "or", "not", "true", "false"];

fn is_judgment_color(c: &str) -> bool {
    c == "judgment" || c == "election"
}

/// Check one `.okf.md` source. `file` is the label used in findings.
///
/// Returns the leaks found (empty = clean), or an error string if the file has
/// no YAML frontmatter or the frontmatter does not parse.
pub fn check(file: &str, source: &str) -> Result<Vec<Finding>, String> {
    let front = frontmatter(source)
        .ok_or_else(|| "no YAML frontmatter (`---` fences) found".to_string())?;
    let doc: Value =
        serde_yaml::from_str(front).map_err(|e| format!("YAML frontmatter parse error: {e}"))?;

    // Pass 1: every name declared judgment/election anywhere in the document.
    let mut judgment = BTreeSet::new();
    collect_judgment_names(&doc, &mut judgment);

    // Pass 2: walk and flag.
    let mut findings = Vec::new();
    walk(&doc, file, String::new(), &judgment, &mut findings);
    Ok(findings)
}

/// Extract the YAML frontmatter (text between the leading `---` fence and the
/// next `---` line). Returns `None` if the source doesn't open with a fence.
fn frontmatter(source: &str) -> Option<&str> {
    let mut lines = source.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    // Byte offset where the body (after the first fence line + newline) begins.
    let start = source.find('\n')? + 1;
    let mut offset = start;
    for line in source[start..].split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Some(&source[start..offset]);
        }
        offset += line.len();
    }
    None
}

/// Pass 1: gather every name declared with `color: judgment|election` — both
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

/// Pass 2: recursively walk, emitting a finding at each offending node.
fn walk(v: &Value, file: &str, path: String, judgment: &BTreeSet<String>, out: &mut Vec<Finding>) {
    match v {
        Value::Mapping(m) => {
            if let Some((test, inputs)) = mechanical_test(m) {
                scan_test(test, &inputs, judgment, file, &path, out);
            }
            if let Some(agg) = aggregation(m) {
                if agg.contains_key("test") || agg.contains_key("formula") {
                    out.push(Finding {
                        file: file.to_string(),
                        path: path.clone(),
                        leak: Leak::FakedAggregation,
                        detail: "judgment-aggregation carries a formula/test — a weighed \
                                 judgment faked as a mechanical combination rule"
                            .to_string(),
                    });
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

/// The aggregation *body* at this node, if this mapping is one — identified by
/// its `factors` + `grounds` content. We match the body, not the
/// `judgment-aggregation:` wrapper key, so the keyed form
/// (`{ judgment-aggregation: { factors, grounds } }`) and the bare form
/// (`{ factors, grounds }`) each detect exactly once (the wrapper is walked
/// through to its body rather than matched itself).
fn aggregation(m: &Mapping) -> Option<&Mapping> {
    if m.contains_key("factors") && m.contains_key("grounds") {
        return Some(m);
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
            out.push(Finding {
                file: file.to_string(),
                path: path.to_string(),
                leak: Leak::JudgmentComputed,
                detail: format!(
                    "mechanical test computes on judgment/election name `{tok}`, \
                     not declared as an opaque input of this test"
                ),
            });
        } else {
            out.push(Finding {
                file: file.to_string(),
                path: path.to_string(),
                leak: Leak::UndeclaredInput,
                detail: format!(
                    "mechanical test references `{tok}`, neither a subject field \
                     (`subject.field`) nor a declared colored input"
                ),
            });
        }
    }
}

/// Split an expression into `(token, is_function_call)` pairs. A token is a
/// maximal run of identifier characters (alphanumeric, `_`, `-`, `.`); it is a
/// function call if the next non-space character is `(`. Kebab-case names stay
/// whole (`fair-value`), while a spaced `a - b` splits into `a` and `b`.
fn tokenize(expr: &str) -> Vec<(String, bool)> {
    let chars: Vec<char> = expr.chars().collect();
    let is_id = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.';
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_id(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && is_id(chars[i]) {
            i += 1;
        }
        let tok: String = chars[start..i].iter().collect();
        let mut j = i;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        let is_call = chars.get(j) == Some(&'(');
        out.push((tok, is_call));
    }
    out
}

/// Render a mapping key as a path segment.
fn key_str(k: &Value) -> String {
    match k {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => "?".to_string(),
    }
}
