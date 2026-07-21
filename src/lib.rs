//! deon static checks over the OKF-frontmatter norm schema (DESIGN §4).
//!
//! Representational-only: parse frontmatter, walk the normal form, flag.
//! Nothing is evaluated (DESIGN §9). Each check lives in its own module:
//!
//! - [`leak`] — check 1, the mechanical edge: no judgment is computed silently
//!   (LEAK-1/2/3).
//! - [`ground`] — check 2, the judgment edge: every judgment *hole* carries a
//!   citation (GROUND-1/2/3).
//! - [`seam`] — check 5, the bottom edge: every norm terminates in a commitment
//!   about plain data, by a well-defined path (SEAM-1/2/3).
//! - [`regime`] — check 6: a norm's mechanized artifacts belong to its regime
//!   (REGIME-1/2).
//! - [`cover`] — check 3: a norm that claims any of its subject's declared
//!   states must claim them all (COVER-1/2; needs a bundle).
//! - [`conflict`] — check 4, the priority edge: a defeat that collides is
//!   reported three-valued — underdetermined while its `binds` is a judgment
//!   hole (CONFLICT-1/2/3).
//!
//! Checks 1 and 2 together enforce deon's core invariant — "no judgment is ever
//! silently evaluated mechanically, *and* every judgment hole carries a
//! citation."

mod conflict;
mod cover;
mod expr;
mod ground;
mod leak;
mod okf;
mod regime;
mod seam;

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
    /// SEAM-1: a norm reaches no commitment about plain data.
    UnreachedSeam,
    /// SEAM-2: a commitment / residual branch constrains no plain data.
    EmptyCommitment,
    /// SEAM-3: a norm mixes the binary and n-ary branch forms.
    MixedBranchForms,
    /// REGIME-1: a norm has no effective regime.
    UndeterminedRegime,
    /// REGIME-2: a `@regime`-stamped artifact does not match its norm's regime.
    CrossRegimeArtifact,
    /// COVER-1: a declared state of the subject that no branch covers.
    UncoveredState,
    /// COVER-2: a `covers:` naming a state the subject does not declare.
    UndeclaredStateClaimed,
    /// CONFLICT-1: a `defeats:` names no norm in the document.
    DanglingDefeat,
    /// CONFLICT-2: a collision whose resolving `binds` is a judgment hole.
    UnderdeterminedConflict,
    /// CONFLICT-3: a collision whose resolving `binds` is mechanical.
    DeterminateConflict,
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
            Rule::UnreachedSeam => "SEAM-1",
            Rule::EmptyCommitment => "SEAM-2",
            Rule::MixedBranchForms => "SEAM-3",
            Rule::UndeterminedRegime => "REGIME-1",
            Rule::CrossRegimeArtifact => "REGIME-2",
            Rule::UncoveredState => "COVER-1",
            Rule::UndeclaredStateClaimed => "COVER-2",
            Rule::DanglingDefeat => "CONFLICT-1",
            Rule::UnderdeterminedConflict => "CONFLICT-2",
            Rule::DeterminateConflict => "CONFLICT-3",
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
            Rule::UnreachedSeam => "unreached seam",
            Rule::EmptyCommitment => "empty commitment",
            Rule::MixedBranchForms => "mixed branch forms",
            Rule::UndeterminedRegime => "undetermined regime",
            Rule::CrossRegimeArtifact => "cross-regime artifact",
            Rule::UncoveredState => "uncovered state",
            Rule::UndeclaredStateClaimed => "undeclared state claimed",
            Rule::DanglingDefeat => "dangling defeat",
            Rule::UnderdeterminedConflict => "underdetermined conflict",
            Rule::DeterminateConflict => "determinate conflict",
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

/// A norm's `cases:` branches as `(path, commitment)` pairs — the n-ary form of
/// the binary `commitment` + `otherwise` shape (DESIGN §3). A case without a
/// `when:` is the residual, exactly what `otherwise` sugars; both are branches
/// here, so every check that reasons about a norm's commitments sees all of
/// them. Returns empty for a norm written in the binary form.
pub(crate) fn cases(norm: &Value) -> Vec<(String, Option<&Value>)> {
    match norm.get("cases") {
        Some(Value::Sequence(s)) => s
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("cases[{i}]"), c.get("commitment")))
            .collect(),
        _ => Vec::new(),
    }
}

/// The string value at `map[key]`, if `map` is a mapping with a string there.
pub(crate) fn str_field(map: &Value, key: &str) -> Option<String> {
    match map.get(key) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
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
/// GROUND-3 (anchor resolution) and coverage (check 3) need an OKF bundle and
/// are *not* included here; call [`check_with_okf`] with an [`Okf`] for those.
pub fn check(file: &str, source: &str) -> Result<Vec<Finding>, String> {
    let doc = parse(source)?;
    let mut findings = Vec::new();
    leak::check(&doc, file, &mut findings);
    ground::structural(&doc, file, &mut findings);
    seam::check(&doc, file, &mut findings);
    regime::check(&doc, file, &mut findings);
    conflict::check(&doc, file, &mut findings);
    Ok(findings)
}

/// Run the checks that need an OKF bundle: GROUND-3 (anchor resolution) and
/// check 3 (coverage, COVER-1/2, which reads the bundle's subject state
/// spaces). Kept separate from [`check`] so a caller running both does not
/// double-report the always-on findings.
pub fn check_with_okf(file: &str, source: &str, okf: &Okf) -> Result<Vec<Finding>, String> {
    let doc = parse(source)?;
    let mut findings = Vec::new();
    ground::anchors(&doc, file, okf, &mut findings);
    cover::check(&doc, file, okf, &mut findings);
    Ok(findings)
}

/// Parse the YAML frontmatter of an `.okf.md` source into a value tree.
fn parse(source: &str) -> Result<Value, String> {
    let front = frontmatter(source)
        .ok_or_else(|| "no YAML frontmatter (`---` fences) found".to_string())?;
    serde_yaml::from_str(front).map_err(|e| format!("YAML frontmatter parse error: {e}"))
}

/// Extract the YAML frontmatter (also used to read an OKF concept file's own
/// frontmatter; see [`okf`]). (text between the leading `---` fence and the
/// next `---` line). Returns `None` if the source doesn't open with a fence.
pub(crate) fn frontmatter(source: &str) -> Option<&str> {
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
