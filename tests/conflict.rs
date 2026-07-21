//! Acceptance tests for the conditional-conflict check (issue #11, check 4).
//!
//! Green: the seed norms carry defeat edges that resolve cleanly — the rev-rec
//! seed's judgment-bound `var-consideration-constraint` modifies a field its
//! target does not commit, and the lease seed's election commits a disjoint
//! field — so neither is a conflict. Red: a fixture whose edges trip all three
//! rules, and whose two silent edges pin what a conflict is *not*.

use std::path::PathBuf;

use deon_check::{check, Finding, Rule};

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn conflict_findings(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| {
            matches!(
                f.rule,
                Rule::DanglingDefeat
                    | Rule::UnderdeterminedConflict
                    | Rule::DeterminateConflict
                    | Rule::UncoloredPriority
            )
        })
        .collect()
}

fn render(findings: &[&Finding]) -> String {
    findings
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Green: the seeds' defeat edges are well-formed and collide on nothing.
#[test]
fn seed_defeat_edges_are_conflict_free() {
    for seed in [
        "examples/revenue-recognition-timing.okf.md",
        "examples/lease-classification.okf.md",
    ] {
        let findings = check(seed, &read(seed)).expect("seed parses");
        let c = conflict_findings(&findings);
        assert!(
            c.is_empty(),
            "expected 0 conflict findings in {seed}, got:\n{}",
            render(&c)
        );
    }
}

/// Red: the fixture trips each rule once — and stays silent on the disjoint and
/// unconditional edges.
#[test]
fn conflicting_fixture_trips_each_rule_once() {
    let rel = "tests/fixtures/conflicting.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let c = conflict_findings(&findings);

    assert_eq!(
        c.len(),
        3,
        "expected 3 conflict findings, got:\n{}",
        render(&c)
    );

    let c2 = c
        .iter()
        .find(|f| f.rule == Rule::UnderdeterminedConflict)
        .expect("CONFLICT-2");
    assert_eq!(c2.path, "norms[1].defeats");
    assert!(
        c2.detail.contains("underdetermined(highly-uncertain)"),
        "CONFLICT-2 reports the ungrounded predicate: {}",
        c2.detail
    );
    assert!(
        c2.detail.contains("`timing`") && c2.detail.contains("over-time"),
        "CONFLICT-2 locates the colliding field: {}",
        c2.detail
    );

    let c3 = c
        .iter()
        .find(|f| f.rule == Rule::DeterminateConflict)
        .expect("CONFLICT-3");
    assert_eq!(c3.path, "norms[2].defeats");
    assert!(c3.detail.contains("past-cutoff"), "{}", c3.detail);

    let c1 = c
        .iter()
        .find(|f| f.rule == Rule::DanglingDefeat)
        .expect("CONFLICT-1");
    assert_eq!(c1.path, "norms[3].defeats");
    assert!(c1.detail.contains("no-such-norm"), "{}", c1.detail);
}

/// Conditional conflict reaches inside the n-ary `cases:` form: a defeater that
/// collides only with a *middle* case is still a collision, and its judgment
/// `binds` makes it underdetermined rather than a static contradiction.
#[test]
fn conflict_collides_against_a_middle_case() {
    let rel = "tests/fixtures/three-case.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let c = conflict_findings(&findings);

    assert_eq!(
        c.len(),
        1,
        "expected 1 conflict finding, got:\n{}",
        render(&c)
    );
    assert_eq!(c[0].rule, Rule::UnderdeterminedConflict);
    assert_eq!(c[0].path, "norms[2].defeats");
    assert!(
        c[0].detail.contains("underdetermined(control-retained)")
            && c[0].detail.contains("point-in-time"),
        "collides on the middle case: {}",
        c[0].detail
    );
}

/// A `defeats:` the checker cannot read must be reported, not skipped — skipping
/// it disables the whole check for that edge. The list form is meaningful (one
/// norm may defeat several), so it is accepted and each target checked; an
/// uncolored `binds` is CONFLICT-4, never "decidable at the seam".
#[test]
fn claim_shapes_are_recognized_not_skipped() {
    let rel = "tests/fixtures/claim-shapes.okf.md";
    let findings = check(rel, &read(rel)).expect("fixture parses");
    let c = conflict_findings(&findings);
    let rendered = render(&c);

    let at = |path: &str, rule: Rule| {
        assert!(
            c.iter().any(|f| f.path == path && f.rule == rule),
            "expected {} at {path} in:\n{rendered}",
            rule.code()
        );
    };

    // One list, two targets, two independent verdicts.
    at("norms[1].defeats[0]", Rule::UnderdeterminedConflict);
    at("norms[1].defeats[1]", Rule::DanglingDefeat);
    // An uncolored priority predicate is not decidable.
    at("norms[2].defeats", Rule::UncoloredPriority);
    assert!(
        !c.iter().any(|f| f.rule == Rule::DeterminateConflict),
        "an uncolored binds must never be reported as determinate:\n{rendered}"
    );
    assert_eq!(
        c.len(),
        3,
        "expected exactly 3 conflict findings:\n{rendered}"
    );
}
