# deon — repo guide for Claude

deon is a **colored deontic norm language + static checker** for accounting
judgment: it captures rules of the shape "you must do X when Y" in a form that is
consistency-checkable, traceable, and auditable. Read the **[README](README.md)**
for the thesis and **[docs/DESIGN.md](docs/DESIGN.md)** for the abstract syntax,
the static checks, and the non-goals. Follow those; this file adds what an agent
needs on top.

## Sibling to Pacioli (developed concurrently)

deon is the judgment-side sibling to
**[Pacioli](https://github.com/ojhermann-org/pacioli)** (`~/pacioli`), and the
two are developed concurrently. Pacioli formalizes the **mechanical** half of
accounting in Lean; deon structures the **judgment** half. The relationship is
load-bearing, not incidental: deon's bottom edge *is* Pacioli's seam — every
well-formed norm terminates in a plain-data commitment the Pacioli kernel checks.
Keep that seam contract in mind when touching either repo.

## Scope: the language, not the norms

deon owns the **language + checker** — the semantics and tooling. It is **not**
a collection of norms and **not** an authority on accounting. Norm *content* lives
beside its cited prose in an OKF judgment bundle; deon *consumes* those bundles
and checks them (`crates/deon/examples/` holds three seed norms until an OKF
bundle exists to hold them). This mirrors Pacioli's own machinery-vs-data
split — keep it sharp: don't grow a catalog of accounting norms in this repo's
core.

## The core discipline: the seam is in the type system

Every predicate is **colored** — `mechanical` (decidable from seam data),
`judgment` (open-textured, must cite where it grounds), or `election` (a
discretionary entity choice). The checker enforces the seam the way Pacioli's
Lean enforces "policy never leaks into types": **no judgment is ever silently
evaluated mechanically, and every judgment hole carries a citation.** This is the
invariant that protects deon's value — do not weaken it.

## Build and verify

- **Representational-only.** The deliverable is static analysis over a normal
  form, and **all six docs/DESIGN.md §4 checks are built** as the `deon-check`
  crate: leak detection (LEAK), grounding completeness (GROUND), coverage
  (COVER), conditional conflict (CONFLICT), termination-at-seam (SEAM), regime
  hygiene (REGIME). There is **no execution engine, no custom syntax, and no
  neural component**, by deliberate choice (docs/DESIGN.md §9). Don't add them
  without going through the design.
- **Run it:** `nix run . -- crates/deon/examples/` for the always-on checks; add
  `--okf <bundle>` for the two that need an OKF concept bundle (GROUND-3 anchor
  resolution and coverage, which reads each subject's declared state space).
  `cargo test` runs the acceptance suite: seeds green, red fixture per check.
- **CI:** `nix flake check` is the single required status check on `main`
  (`.github/rulesets/main.json`, reconciled by `scripts/settings.sh`). It gates
  `cargo fmt --check`, `clippy -D warnings`, and the test suite.
- **Repo settings as code:** `.github/rulesets/main.json` reconciled by
  `scripts/settings.sh` (mirrors Pacioli's approach).

## Landing changes

- `main` is **PR-only** and branch-protected (required owner review + merge
  queue); nothing merges without the owner. `scripts/merge.sh <pr>` lands an
  owner-authored PR via ruleset bypass — the agent runs it **only on the owner's
  explicit ask** (see ~/.claude/CLAUDE.md, Pull-requests → the deon carve-out).
- `merge.sh` gates on green CI, so it lands a PR **without** `--force` once
  `nix flake check` passes; **`--force` (which bypasses that gate) stays
  owner-run/confirm-first.**

## Deletion & creation

- Do **not** weaken the coloring discipline — a `judgment`/`election` predicate
  must never be evaluated as `mechanical`, and every judgment hole must carry a
  `grounds` citation.
- Do **not** add an execution engine, a bespoke syntax, or neural components
  ahead of the design (docs/DESIGN.md §9) — they are deferred on purpose, to be
  earned once the representation is validated.
- Treat `docs/DESIGN.md` and the two `docs/spikes/` notes as the load-bearing
  record of *why* the language is shaped as it is; don't casually rewrite them.
- Keep norm *content* out of the core — it belongs beside cited OKF prose;
  `crates/deon/examples/` is a seed set, not a home for a growing catalog.
