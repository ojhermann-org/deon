//! Acceptance tests for the coverage check (issue #11, check 3).
//!
//! Coverage needs a bundle (the subject's state space is norm content), so it
//! runs only under `--okf` — like GROUND-3. The rev-rec seed's known gap is
//! flagged *as expected*: the seed stays a faithful rendering of spike 1, F5.

use std::path::PathBuf;

use deon_check::{check, check_bundle, check_with_okf, Finding, Okf, Rule};

fn manifest(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    let path = manifest(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn bundle() -> Okf {
    Okf::load(&manifest("tests/fixtures/okf-states")).expect("state-space bundle loads")
}

fn cover_findings(findings: &[Finding]) -> Vec<&Finding> {
    findings
        .iter()
        .filter(|f| matches!(f.rule, Rule::UncoveredState | Rule::UndeclaredStateClaimed))
        .collect()
}

fn render(findings: &[&Finding]) -> String {
    findings
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The headline finding (spike 1, F5): the seed's over-time/point-in-time split
/// hides the "not yet satisfied → recognize nothing" third state, and coverage
/// names it. Left open in the seed on purpose — closing it would hide the very
/// finding the check exists to make.
#[test]
fn seed_gap_is_flagged_as_expected() {
    let rel = "examples/revenue-recognition-timing.okf.md";
    let findings = check_with_okf(rel, &read(rel), &bundle()).expect("seed parses");
    let c = cover_findings(&findings);

    assert_eq!(
        c.len(),
        1,
        "expected exactly the known gap, got:\n{}",
        render(&c)
    );
    assert_eq!(c[0].rule, Rule::UncoveredState);
    assert_eq!(c[0].path, "norms[0]");
    assert!(
        c[0].detail.contains("not-yet-satisfied"),
        "the finding names the missing state: {}",
        c[0].detail
    );
}

/// Coverage is bundle-backed: without `--okf` the seed's gap is invisible, so
/// the always-on run stays clean.
#[test]
fn coverage_is_silent_without_a_bundle() {
    for seed in [
        "examples/revenue-recognition-timing.okf.md",
        "examples/lease-classification.okf.md",
    ] {
        let findings = check(seed, &read(seed)).expect("seed parses");
        assert!(
            cover_findings(&findings).is_empty(),
            "coverage must not run without a bundle ({seed})"
        );
    }
}

/// Red: a gap is named, an undeclared claim is flagged two ways, a fully
/// covering norm is clean — and a norm that claims nothing is not guessed at.
#[test]
fn uncovered_fixture_trips_both_rules() {
    let rel = "tests/fixtures/uncovered.okf.md";
    let findings = check_with_okf(rel, &read(rel), &bundle()).expect("fixture parses");
    let c = cover_findings(&findings);

    assert_eq!(
        c.len(),
        3,
        "expected 3 coverage findings, got:\n{}",
        render(&c)
    );

    // `c1-gap` claims `a` and never claims `b`.
    let gap = c.iter().find(|f| f.rule == Rule::UncoveredState).unwrap();
    assert_eq!(gap.path, "norms[1]");
    assert!(gap.detail.contains('b'), "{}", gap.detail);

    // `c2-undeclared` claims a state the subject does not declare...
    let undeclared: Vec<_> = c
        .iter()
        .filter(|f| f.rule == Rule::UndeclaredStateClaimed)
        .collect();
    assert_eq!(undeclared.len(), 2);
    let z = undeclared
        .iter()
        .find(|f| f.path == "norms[2].cases[2].covers")
        .expect("COVER-2 located at the claiming branch");
    assert!(z.detail.contains("no state declared"), "{}", z.detail);

    // ...and `unknown-subject` claims one for a subject with no state space.
    let x = undeclared
        .iter()
        .find(|f| f.path == "norms[4].covers")
        .expect("COVER-2 for the undeclared subject");
    assert!(x.detail.contains("no state space"), "{}", x.detail);

    // `full-cover` (norms[0]) and `no-claim` (norms[3]) are absent: one covers
    // everything, the other makes no claim to check.
    assert!(
        !c.iter()
            .any(|f| f.path.starts_with("norms[0]") || f.path.starts_with("norms[3]")),
        "a covering norm and a claimless norm must both stay silent:\n{}",
        render(&c)
    );
}

/// The bundle is held to the standard it enforces (issue #18): a state
/// declaration must name a state and cite where it grounds. The well-formed
/// declaration stays silent; each defect is located in the concept file.
#[test]
fn bundle_state_declarations_are_grounded() {
    let okf = Okf::load(&manifest("tests/fixtures/okf-bad-states")).expect("bundle loads");
    let findings = check_bundle(&okf);
    let rendered = findings
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(findings.len(), 4, "expected 4 findings, got:\n{rendered}");
    for (i, rule) in [
        (1, Rule::MissingCitation),
        (2, Rule::InvalidSource),
        (3, Rule::DanglingAnchor),
        (4, Rule::MalformedState),
    ] {
        let f = findings
            .iter()
            .find(|f| f.path == format!("subjects.thing.states[{i}]"))
            .unwrap_or_else(|| panic!("no finding at states[{i}] in:\n{rendered}"));
        assert_eq!(f.rule, rule, "at states[{i}]");
        assert!(
            f.file.ends_with("concepts.md"),
            "located in the bundle: {}",
            f.file
        );
    }

    // The malformed declaration also vanishes from the state space — which is
    // exactly why COVER-3 has to exist.
    assert!(
        !findings.iter().any(|f| f.path.ends_with("states[0]")),
        "the well-formed declaration must stay silent:\n{rendered}"
    );
}

/// The coverage bundle grounds its own state space, so checking it is clean.
#[test]
fn state_space_bundle_grounds_itself() {
    let findings = check_bundle(&bundle());
    assert!(
        findings.is_empty(),
        "the fixture bundle must ground its own claims, got:\n{}",
        findings
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
