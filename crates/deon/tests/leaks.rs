//! Acceptance tests for the leak-detection check (issue #2).
//!
//! "A checker you've only seen say 'clean' isn't a checker" — so we assert both
//! a green case (the honestly-authored seed norms → 0 leaks) and a red case (a
//! deliberately-leaky fixture → 3 leaks, each located and of the right kind).

use std::path::PathBuf;

use deon_check::{check, Rule};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Green case: every seed norm is authored honestly and must be clean across
/// all always-on checks (leak + grounding GROUND-1/2).
#[test]
fn seed_norms_are_clean() {
    for seed in [
        "examples/revenue-recognition-timing.okf.md",
        "examples/lease-classification.okf.md",
    ] {
        let findings = check(seed, &read(seed)).expect("seed parses");
        assert!(
            findings.is_empty(),
            "expected 0 findings in {seed}, got {}:\n{}",
            findings.len(),
            findings
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Red case: the fixture trips each rule exactly once, and each finding is
/// located at the node that caused it.
#[test]
fn leaky_fixture_trips_each_rule_once() {
    let rel = "tests/fixtures/leaky.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");

    assert_eq!(
        findings.len(),
        3,
        "expected exactly 3 leaks, got:\n{}",
        findings
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Exactly one of each kind.
    for kind in [
        Rule::JudgmentComputed,
        Rule::UndeclaredInput,
        Rule::FakedAggregation,
    ] {
        assert_eq!(
            findings.iter().filter(|f| f.rule == kind).count(),
            1,
            "expected exactly one {} finding",
            kind.code()
        );
    }

    // Each finding is located at (a path under) the norm that caused it.
    let at = |kind: Rule| findings.iter().find(|f| f.rule == kind).unwrap();
    assert!(
        at(Rule::JudgmentComputed)
            .path
            .contains("leak1-judgment-computed")
            || at(Rule::JudgmentComputed).path.starts_with("norms[0]")
    );
    assert!(at(Rule::UndeclaredInput).path.starts_with("norms[1]"));
    assert!(at(Rule::FakedAggregation).path.starts_with("norms[2]"));

    // The judgment-computed leak names the offending judgment predicate.
    assert!(at(Rule::JudgmentComputed).detail.contains("is-material"));
    // The undeclared-input leak names the offending bare token.
    assert!(at(Rule::UndeclaredInput).detail.contains("benchmark"));
}

/// A file without frontmatter is an error, not a silent pass.
#[test]
fn missing_frontmatter_is_an_error() {
    assert!(check("x.md", "# just prose\n").is_err());
}

/// The near-miss forms: the small edits that used to make a rule stop applying
/// instead of making it fire. Each of these was silent before recognition was
/// keyed on the weakest signal that a node was intended.
#[test]
fn near_miss_forms_are_caught() {
    let rel = "tests/fixtures/near-miss.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let rendered = findings
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let at = |path: &str, rule: Rule| {
        assert!(
            findings.iter().any(|f| f.path == path && f.rule == rule),
            "expected {} at {path} in:\n{rendered}",
            rule.code()
        );
    };

    // A commitment field that says nothing about its side of the seam.
    at("norms[3].commitment.method", Rule::UncoloredCommitmentField);
    // A field access on a judgment name is still computing on the judgment.
    at("norms[0].antecedent.all-of[1]", Rule::JudgmentComputed);
    // A dotted name rooted outside the norm's subject is undeclared data.
    at("norms[1].antecedent.all-of[0]", Rule::UndeclaredInput);
    // An input entry that names no color is not a declaration.
    at(
        "norms[2].antecedent.is-big.inputs.benchmark",
        Rule::UndeclaredInput,
    );
    // An uncited aggregation is still an aggregation: it must trip BOTH the
    // rule that demands the citation and the one that catches the formula.
    at(
        "norms[4].antecedent.weigh.judgment-aggregation",
        Rule::FakedAggregation,
    );
    at(
        "norms[4].antecedent.weigh.judgment-aggregation",
        Rule::MissingCitation,
    );

    assert_eq!(
        findings.len(),
        6,
        "expected exactly 6 findings:\n{rendered}"
    );
}

/// Deleting a citation must make the checker say *more*, not less. This is the
/// invariant behind the recognition fix: what a rule checks for must never also
/// be what makes the rule apply, or the check gets weaker exactly as the input
/// gets more dishonest.
#[test]
fn removing_a_citation_cannot_silence_a_rule() {
    let cited = "\
---
norms:
  - id: n
    subject: thing
    antecedent:
      weigh:
        judgment-aggregation:
          factors: [a, b]
          grounds: { ref: \"#w\", source: standard-criterion }
          test: \"0.5*a + 0.5*b >= threshold\"
    commitment: { flag: true }
---
# body
";
    let uncited = cited
        .lines()
        .filter(|l| !l.trim_start().starts_with("grounds:"))
        .collect::<Vec<_>>()
        .join("\n");

    let with = check("cited.okf.md", cited).expect("parses");
    let without = check("uncited.okf.md", &uncited).expect("parses");

    assert!(
        !with.is_empty(),
        "control: the cited aggregation still trips LEAK-3 for its formula"
    );
    assert!(
        without.len() > with.len(),
        "deleting `grounds:` produced {} findings, down from {} — the check got \
         weaker on the more dishonest input:\n{}",
        without.len(),
        with.len(),
        without
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        without.iter().any(|f| f.rule == Rule::MissingCitation),
        "the deleted citation must itself be the new finding"
    );
}
