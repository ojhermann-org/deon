//! Check 4 — conditional conflict (DESIGN §4, check 4).
//!
//! Priority is itself colored (DESIGN §3): a defeat edge carries a `binds`
//! predicate, and that predicate may be a judgment hole. So when two norms
//! collide and the edge that would resolve them binds on a judgment, the honest
//! static report is **not** "contradiction" — it is _underdetermined until
//! grounded_. This check is that report, three-valued:
//!
//! - **CONFLICT-1 — dangling defeat.** `defeats:` names no norm in the document,
//!   so the priority edge points nowhere.
//! - **CONFLICT-2 — underdetermined conflict.** A collision resolved by a
//!   `judgment`/`election`-colored `binds` — reported as
//!   `underdetermined(<predicate>)`, never as a static contradiction.
//! - **CONFLICT-3 — determinate conflict.** The same collision where `binds` is
//!   mechanical: decidable at the seam, so it is reported as a real conflict.
//!
//! A **collision** is representational, not evaluated (DESIGN §9): the defeater
//! and the defeated both constrain the same commitment field with different
//! values. Two deliberate non-flags follow from that definition:
//!
//! - A defeat edge with **no collision** is silent — a `defeats:` whose norms
//!   constrain disjoint fields (the lease seed's election commits `capitalize`,
//!   the norm it defeats commits `classification`) has nothing to resolve.
//! - A defeat edge with **no `binds`** is silent even on a collision: priority is
//!   unconditional there, so it is settled, not conditional.
//!
//! A `modifies:` naming a field the defeated norm does not commit is likewise
//! **not** flagged: the rev-rec seed's `commitment.amount` is deliberately
//! absent upstream because the arithmetic is downstream Lean's (DESIGN §3), so
//! treating it as dangling would punish a correct norm.

use serde_yaml::Value;

use crate::{cases, is_judgment_color, key_str, str_field, Finding, Rule};

/// Run check 4 over the parsed document, appending findings.
pub(crate) fn check(doc: &Value, file: &str, out: &mut Vec<Finding>) {
    let Some(Value::Sequence(norms)) = doc.get("norms") else {
        return;
    };
    let ids: Vec<Option<String>> = norms.iter().map(|n| str_field(n, "id")).collect();

    for (i, norm) in norms.iter().enumerate() {
        let Some(target) = str_field(norm, "defeats") else {
            continue;
        };
        let path = format!("norms[{i}].defeats");
        let here = ids[i].clone().unwrap_or_else(|| format!("norms[{i}]"));

        let Some(j) = ids
            .iter()
            .position(|id| id.as_deref() == Some(target.as_str()))
        else {
            out.push(Finding::new(
                file,
                &path,
                Rule::DanglingDefeat,
                format!(
                    "`defeats: {target}` names no norm in this document — the priority \
                     edge points nowhere"
                ),
            ));
            continue;
        };

        let collisions = collisions(norm, &norms[j]);
        if collisions.is_empty() {
            continue;
        }
        // No `binds` at all: priority is unconditional, so the collision is
        // already settled by the edge itself.
        let Some(binds) = norm.get("binds") else {
            continue;
        };
        let (predicate, color) = binding(binds);
        let fields = collisions.join(", ");

        match color.as_deref() {
            Some(c) if is_judgment_color(c) => out.push(Finding::new(
                file,
                &path,
                Rule::UnderdeterminedConflict,
                format!(
                    "underdetermined({predicate}) — `{here}` defeats `{target}` on {fields}, \
                     bound by a {c} predicate: the conflict cannot be resolved until that \
                     hole is grounded, so it is not a static contradiction (DESIGN §4.4)"
                ),
            )),
            other => out.push(Finding::new(
                file,
                &path,
                Rule::DeterminateConflict,
                format!(
                    "`{here}` defeats `{target}` on {fields}, bound by {} predicate \
                     `{predicate}` — the collision is real and decidable at the seam",
                    match other {
                        Some(c) => format!("a {c}"),
                        None => "an uncolored".to_string(),
                    }
                ),
            )),
        }
    }
}

/// The `binds` predicate's name and color. `binds` appears either as the bare
/// predicate name of the abstract grammar (`binds: <predicate>`) or as the
/// colored mapping the seeds render (`binds: { predicate, color, grounds }`).
fn binding(binds: &Value) -> (String, Option<String>) {
    match binds {
        Value::String(s) => (s.clone(), None),
        _ => (
            str_field(binds, "predicate").unwrap_or_else(|| "?".to_string()),
            str_field(binds, "color"),
        ),
    }
}

/// The commitment fields on which `a` and `b` constrain plain data differently,
/// rendered as `` `field` (`x` vs `y`) `` for the report.
fn collisions(a: &Value, b: &Value) -> Vec<String> {
    let (a, b) = (constrained(a), constrained(b));
    let mut out = Vec::new();
    for (field, av) in &a {
        for (bfield, bv) in &b {
            if field == bfield && av != bv {
                out.push(format!("`{field}` (`{}` vs `{}`)", render(av), render(bv)));
                break;
            }
        }
    }
    out
}

/// Every `(field, value)` a norm constrains at the seam: its `commitment`, its
/// residual `otherwise.commitment`, every `cases[i].commitment`, and any
/// `modifies` (whose `commitment.<f>` keys are normalized to `<f>`, since they
/// name a field of the defeated norm's commitment). `note` is prose, not a
/// constraint, so it is skipped.
///
/// Cases are gathered like any other branch: a collision against the *middle*
/// case of an n-ary norm is as real as one against its residual.
fn constrained(norm: &Value) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    let sources = [
        (norm.get("commitment"), false),
        (
            norm.get("otherwise").and_then(|o| o.get("commitment")),
            false,
        ),
        (norm.get("modifies"), true),
    ]
    .into_iter()
    .chain(cases(norm).into_iter().map(|(_, c)| (c, false)));

    for (source, strip) in sources {
        let Some(Value::Mapping(m)) = source else {
            continue;
        };
        for (k, v) in m {
            let mut field = key_str(k);
            if strip {
                field = field
                    .strip_prefix("commitment.")
                    .unwrap_or(&field)
                    .to_string();
            }
            if field != "note" {
                out.push((field, v.clone()));
            }
        }
    }
    out
}

/// Render a constrained value for a finding's detail line.
fn render(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => "<structured>".to_string(),
    }
}
