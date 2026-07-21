//! Acceptance tests for the grounding-completeness check (issue #10).
//!
//! Green: the seed norms are grounded honestly (every criterion cited + typed;
//! judgment inputs typed by `source`). Red: a deliberately-ungrounded fixture
//! trips GROUND-1 and GROUND-2 always, and GROUND-3 under `--okf`.

use std::path::PathBuf;

use deon_check::{check, check_with_okf, Okf, Rule};

fn manifest(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    let path = manifest(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn ground_findings(findings: &[deon_check::Finding]) -> Vec<&deon_check::Finding> {
    findings
        .iter()
        .filter(|f| {
            matches!(
                f.rule,
                Rule::MissingCitation | Rule::InvalidSource | Rule::DanglingAnchor
            )
        })
        .collect()
}

/// Green: the seeds carry no grounding findings. In particular the judgment
/// *inputs* (`economic-life`, `fair-value`, `rate`) are grounded by their
/// `source` type alone — they are values, not criteria (issue #10 decision), so
/// their lack of a `grounds.ref` must NOT flag.
#[test]
fn seed_norms_are_grounded() {
    for seed in [
        "examples/revenue-recognition-timing.okf.md",
        "examples/lease-classification.okf.md",
    ] {
        let findings = check(seed, &read(seed)).expect("seed parses");
        let g = ground_findings(&findings);
        assert!(
            g.is_empty(),
            "expected 0 grounding findings in {seed}, got:\n{}",
            g.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Red (always-on): the fixture trips GROUND-1 and GROUND-2 once each, located.
#[test]
fn ungrounded_fixture_trips_structural_rules() {
    let rel = "tests/fixtures/ungrounded.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let g = ground_findings(&findings);

    assert_eq!(
        g.len(),
        3,
        "expected exactly 3 structural grounding findings, got:\n{}",
        g.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let g1 = g.iter().find(|f| f.rule == Rule::MissingCitation).unwrap();
    let g2 = g.iter().find(|f| f.rule == Rule::InvalidSource).unwrap();
    assert!(g1.path.starts_with("norms[1]"), "GROUND-1 at {}", g1.path);
    assert!(g2.path.starts_with("norms[2]"), "GROUND-2 at {}", g2.path);
    assert!(g2.detail.contains("vibes"));

    // An `election` must ground in entity policy — the one thing that makes the
    // color distinct from `judgment` anywhere in the checker.
    let election = g
        .iter()
        .find(|f| f.path.starts_with("norms[3]"))
        .expect("GROUND-2 for the election grounded in the standard");
    assert_eq!(election.rule, Rule::InvalidSource);
    assert!(
        election.detail.contains("entity-election"),
        "{}",
        election.detail
    );
}

/// Red (with --okf): the well-formed-but-unresolvable ref trips GROUND-3, while
/// the resolvable control (`g0-resolves`) does not.
#[test]
fn dangling_anchor_trips_only_under_okf() {
    let rel = "tests/fixtures/ungrounded.okf.md";
    let src = read(rel);
    let okf = Okf::load(&manifest("tests/fixtures/okf")).expect("okf bundle loads");
    assert_eq!(okf.len(), 1, "fixture bundle should declare one anchor");

    let bundle_findings = check_with_okf(rel, &src, &okf).expect("fixture parses");
    let anchors = ground_findings(&bundle_findings);
    assert_eq!(
        anchors.len(),
        1,
        "expected exactly one GROUND-3, got:\n{}",
        anchors
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let f = anchors[0];
    assert_eq!(f.rule, Rule::DanglingAnchor);
    assert!(f.path.starts_with("norms[4]"), "GROUND-3 at {}", f.path);
    assert!(f.detail.contains("#does-not-exist"));
}

/// The issue #10 decision, unit-form: a judgment *input* is grounded by its
/// `source` type (no `ref` needed); a missing/invalid source still flags.
#[test]
fn judgment_input_is_grounded_by_source() {
    let ok = "\
---
norms:
  - id: n
    subject: thing
    antecedent:
      m:
        mechanical:
          test: \"thing.a / est >= threshold\"
          inputs: { est: { color: judgment, source: world-fact } }
    commitment: { flag: true }
---
# body
";
    assert!(
        ground_findings(&check("ok.okf.md", ok).unwrap()).is_empty(),
        "a judgment input with a valid source must not flag"
    );

    let bad = ok.replace("source: world-fact", "source: hunch");
    let g = check("bad.okf.md", &bad).unwrap();
    let g = ground_findings(&g);
    assert_eq!(
        g.len(),
        1,
        "a judgment input with a bad source must flag once"
    );
    assert_eq!(g[0].rule, Rule::InvalidSource);
}
