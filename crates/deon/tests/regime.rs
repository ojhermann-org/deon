//! Acceptance tests for the regime-hygiene check (issue #11, check 6).
//!
//! Green: the seed norms are regime-coherent — every norm has an effective
//! regime (own or inherited) and every threshold's `@regime` matches it,
//! including the `IFRS-16` override in the lease seed. Red: a fixture with a
//! cross-regime threshold and a norm with no effective regime.

use std::path::PathBuf;

use deon_check::{check, Finding, Rule};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn regime_findings(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| matches!(f.rule, Rule::UndeterminedRegime | Rule::CrossRegimeArtifact))
        .collect()
}

/// Green: the seeds are regime-coherent — including the lease seed's `IFRS-16`
/// override and its `@IFRS-16` threshold, and its deliberate cross-regime
/// `defeats` edge (which this check does not flag).
#[test]
fn seed_norms_are_regime_coherent() {
    for seed in [
        "examples/revenue-recognition-timing.okf.md",
        "examples/lease-classification.okf.md",
    ] {
        let findings = check(seed, &read(seed)).expect("seed parses");
        let r = regime_findings(&findings);
        assert!(
            r.is_empty(),
            "expected 0 regime findings in {seed}, got:\n{}",
            r.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Red: the fixture trips REGIME-1 and REGIME-2 once each, each located.
#[test]
fn wrong_regime_fixture_trips_both_rules() {
    let rel = "tests/fixtures/wrong-regime.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let r = regime_findings(&findings);

    assert_eq!(
        r.len(),
        2,
        "expected 2 regime findings, got:\n{}",
        r.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );

    let r2 = r
        .iter()
        .find(|f| f.rule == Rule::CrossRegimeArtifact)
        .unwrap();
    let r1 = r
        .iter()
        .find(|f| f.rule == Rule::UndeterminedRegime)
        .unwrap();
    assert!(r2.path.starts_with("norms[0]"), "REGIME-2 at {}", r2.path);
    assert!(r2.detail.contains("IFRS-16") && r2.detail.contains("ASC-840"));
    assert_eq!(r1.path, "norms[1]");
}
