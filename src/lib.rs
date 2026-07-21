//! deon static checks over the OKF-frontmatter norm schema (DESIGN §4).
//!
//! Representational-only: parse frontmatter, walk the normal form, flag.
//! Nothing is evaluated (DESIGN §9). Each check lives in its own module:
//!
//! - [`leak`] — check 1, the mechanical edge: no judgment is computed silently
//!   (LEAK-1/2/3).
//! - [`ground`] — check 2, the judgment edge: every judgment *hole* carries a
//!   citation (GROUND-1/2/3).
//!
//! The two together enforce deon's core invariant — "no judgment is ever
//! silently evaluated mechanically, *and* every judgment hole carries a
//! citation."

mod expr;
mod ground;
mod leak;
mod okf;

pub use okf::Okf;

use std::fmt;

use serde_yaml::{Mapping, Value};

/// Every rule the checker can report, across all checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// LEAK-1: a mechanical test computes on a judgment/election name.
    JudgmentComputed,
    /// LEAK-2: a mechanical test references an undeclared / uncolored input.
    UndeclaredInput,
    /// LEAK-3: a judgment-aggregation carries a formula/test.
    FakedAggregation,
    /// GROUND-1: a judgment hole (a criterion) carries no citation.
    MissingCitation,
    /// GROUND-2: a citation's `source` type is absent or not one of the four.
    InvalidSource,
    /// GROUND-3: a citation's `ref` does not resolve in the OKF bundle.
    DanglingAnchor,
}

impl Rule {
    /// Stable short code, e.g. `LEAK-1` / `GROUND-2`.
    pub fn code(self) -> &'static str {
        match self {
            Rule::JudgmentComputed => "LEAK-1",
            Rule::UndeclaredInput => "LEAK-2",
            Rule::FakedAggregation => "LEAK-3",
            Rule::MissingCitation => "GROUND-1",
            Rule::InvalidSource => "GROUND-2",
            Rule::DanglingAnchor => "GROUND-3",
        }
    }

    /// Human-readable rule name.
    pub fn title(self) -> &'static str {
        match self {
            Rule::JudgmentComputed => "judgment computed",
            Rule::UndeclaredInput => "undeclared/uncolored input",
            Rule::FakedAggregation => "faked aggregation",
            Rule::MissingCitation => "missing citation",
            Rule::InvalidSource => "invalid source type",
            Rule::DanglingAnchor => "dangling anchor",
        }
    }
}

/// A located finding: which file, which node, which rule, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Source file the finding was found in.
    pub file: String,
    /// Node path into the frontmatter, e.g. `norms[0].antecedent.over-time`.
    pub path: String,
    /// Which rule tripped.
    pub rule: Rule,
    /// One-line explanation.
    pub detail: String,
}

impl Finding {
    pub(crate) fn new(file: &str, path: &str, rule: Rule, detail: String) -> Self {
        Finding {
            file: file.to_string(),
            path: path.to_string(),
            rule,
            detail,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}\t{} ({}): {}",
            self.file,
            self.path,
            self.rule.code(),
            self.rule.title(),
            self.detail
        )
    }
}

/// The declared `grounds.source` types (DESIGN §3). A citation must be typed as
/// one of these.
pub(crate) const SOURCE_TYPES: &[&str] = &[
    "standard-criterion",
    "world-fact",
    "legal-fact",
    "entity-election",
];

/// True for a color on the judgment side of the seam.
pub(crate) fn is_judgment_color(c: &str) -> bool {
    c == "judgment" || c == "election"
}

/// The aggregation *body* at this node, if this mapping is one — identified by
/// its `factors` + `grounds` content (the wrapper `judgment-aggregation:` key is
/// walked through to its body, so keyed and bare forms detect once).
pub(crate) fn aggregation(m: &Mapping) -> Option<&Mapping> {
    if m.contains_key("factors") && m.contains_key("grounds") {
        Some(m)
    } else {
        None
    }
}

/// Render a mapping key as a path segment.
pub(crate) fn key_str(k: &Value) -> String {
    match k {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => "?".to_string(),
    }
}

/// Run the always-on static checks (check 1 + check 2's structural rules
/// GROUND-1/2) over one `.okf.md` source. `file` labels the findings.
///
/// GROUND-3 (anchor resolution) needs an OKF bundle and is *not* included here;
/// call [`check_anchors`] with an [`Okf`] for that.
pub fn check(file: &str, source: &str) -> Result<Vec<Finding>, String> {
    let doc = parse(source)?;
    let mut findings = Vec::new();
    leak::check(&doc, file, &mut findings);
    ground::structural(&doc, file, &mut findings);
    Ok(findings)
}

/// Run GROUND-3 only: every well-formed citation's `ref` must resolve to an
/// anchor in `okf`. Kept separate from [`check`] so a caller running both does
/// not double-report the structural findings.
pub fn check_anchors(file: &str, source: &str, okf: &Okf) -> Result<Vec<Finding>, String> {
    let doc = parse(source)?;
    let mut findings = Vec::new();
    ground::anchors(&doc, file, okf, &mut findings);
    Ok(findings)
}

/// Parse the YAML frontmatter of an `.okf.md` source into a value tree.
fn parse(source: &str) -> Result<Value, String> {
    let front = frontmatter(source)
        .ok_or_else(|| "no YAML frontmatter (`---` fences) found".to_string())?;
    serde_yaml::from_str(front).map_err(|e| format!("YAML frontmatter parse error: {e}"))
}

/// Extract the YAML frontmatter (text between the leading `---` fence and the
/// next `---` line). Returns `None` if the source doesn't open with a fence.
fn frontmatter(source: &str) -> Option<&str> {
    let mut lines = source.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }
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
