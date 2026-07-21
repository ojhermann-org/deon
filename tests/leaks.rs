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
