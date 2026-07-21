//! Acceptance tests for the termination-at-seam check (issue #11, check 5).
//!
//! Green: every seed norm terminates — a non-empty `commitment` (the
//! `var-consideration-constraint` defeat reaches the seam via `modifies`), and
//! every `otherwise` residual branch carries a commitment. Red: a fixture that
//! reaches no seam / carries empty commitments.

use std::path::PathBuf;

use deon_check::{check, Finding, Rule};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn seam_findings(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| matches!(f.rule, Rule::UnreachedSeam | Rule::EmptyCommitment))
        .collect()
}

/// Green: the seeds all terminate at the seam (including the modifies-only
/// defeat entry and the otherwise branches).
#[test]
fn seed_norms_terminate() {
    for seed in [
        "examples/revenue-recognition-timing.okf.md",
        "examples/lease-classification.okf.md",
    ] {
        let findings = check(seed, &read(seed)).expect("seed parses");
        let s = seam_findings(&findings);
        assert!(
            s.is_empty(),
            "expected 0 termination findings in {seed}, got:\n{}",
            s.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Red: the fixture trips SEAM-1 once and SEAM-2 twice, each located.
#[test]
fn unterminated_fixture_trips_seam_rules() {
    let rel = "tests/fixtures/unterminated.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let s = seam_findings(&findings);

    assert_eq!(
        s.len(),
        3,
        "expected 3 termination findings, got:\n{}",
        s.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );

    let seam1: Vec<_> = s.iter().filter(|f| f.rule == Rule::UnreachedSeam).collect();
    let seam2: Vec<_> = s
        .iter()
        .filter(|f| f.rule == Rule::EmptyCommitment)
        .collect();
    assert_eq!(seam1.len(), 1, "expected one SEAM-1");
    assert_eq!(seam2.len(), 2, "expected two SEAM-2");

    assert_eq!(seam1[0].path, "norms[0]");
    // empty commitment at norms[1].commitment; dead-end otherwise at norms[2].otherwise
    assert!(seam2.iter().any(|f| f.path == "norms[1].commitment"));
    assert!(seam2.iter().any(|f| f.path == "norms[2].otherwise"));
}

/// The n-ary `cases:` form: a well-formed three-case norm terminates, and
/// SEAM-2 reaches *inside* the form — a case that commits nothing is a branch
/// that leads nowhere, exactly as a dead-end `otherwise` is.
#[test]
fn cases_form_terminates_and_is_reached() {
    let rel = "tests/fixtures/three-case.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let s = seam_findings(&findings);

    assert_eq!(
        s.len(),
        1,
        "expected 1 termination finding, got:\n{}",
        s.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(s[0].rule, Rule::EmptyCommitment);
    assert_eq!(s[0].path, "norms[1].cases[1]");
}
