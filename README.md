# deon — a colored deontic norm language for accounting judgment

From Greek _déon_, "that which is binding" — the root of _deontic_. A small
language and static checker for **accounting judgments and their consequences**,
sitting between curated judgment prose (the **Open Knowledge Format**,
[OKF][okf-spec]) and a verified mechanical core (the
[Pacioli](https://github.com/ojhermann-org/pacioli) Lean library). It captures rules
of the shape "you must do X when Y" — obligations, permissions, prohibitions — in
a form that is **consistency-checkable, traceable, and auditable**, while every
actual judgment stays an explicit, cited hole rather than a hardcoded policy.

## What it is (and is not)

deon is the **language and its checker** — the semantics and tooling. It is _not_
a collection of norms and it is _not_ an authority on accounting. Norm _content_
lives beside its cited prose in an OKF judgment bundle; deon _consumes_ those
bundles and checks them. It ships a small seed set of worked examples
([`examples/`](examples/) — revenue-recognition timing, lease classification)
until an OKF bundle exists to hold them.

This mirrors [the split][pacioli-split] its sibling **Pacioli** keeps on the
mechanical side: Pacioli owns the verified _machinery_, not the accounting data;
deon owns the _norm language_, not the norms.

## Why it exists

Lean proves the mechanics are right _given_ the inputs. OKF prose says _what_ the
inputs should be and _why_. Neither shows that a proposed set of inputs is
**justified** — nor names the chain of judgments it rests on. That gap is what
deon fills:

- **consistency** — conflicting obligations, coverage gaps, dead rules, found
  statically;
- **traceability** — from a proposed seam input back to the exact judgments
  beneath it;
- **auditable consequence-propagation** — "this obligation follows, and it hinges
  on judgments P and Q, currently ungrounded."

deon _assists_ Lean + OKF; it adds no new authority over either.

## The core idea: the seam is in the type system

Every predicate is **colored** — `mechanical` (decidable from seam data),
`judgment` (open-textured, must cite where it grounds), or `election` (a
discretionary entity choice). The checker enforces the seam the way Lean enforces
["policy never leaks into types"][pacioli-seam]: no judgment is ever silently evaluated
mechanically, and every judgment hole carries a citation. A standard's logical
_structure_ is mechanical; its _criteria_ are judgment — and deon lets mechanical
connectives compose over judgment atoms so both live in one rule.

Full abstract syntax and static checks: **[docs/DESIGN.md](docs/DESIGN.md)**. The
two worked spikes that this design rests on: **[docs/spikes/](docs/spikes/)**.

## Bounded on both ends

- **Top** grounds in OKF prose + citations; prose stays authoritative.
- **Bottom** grounds in the [Pacioli Lean seam][pacioli-seam]: every well-formed
  norm terminates in a commitment about plain data the kernel checks. A norm that
  never constrains seam data is malformed.

## Scope discipline (borrowed from Pacioli, deliberately)

- **Representational first.** Static analysis over a normal form is the first
  deliverable. A facts→verdict execution engine is a later, earned commitment.
- **No custom syntax yet.** Norms begin as a disciplined schema inside OKF
  frontmatter, beside their prose. The bespoke DSL syntax is a later ergonomic
  layer over a validated semantics.
- **Neural stays at the edges.** Extraction (prose → candidate norms) and
  grounding (resolving open-textured holes at evaluation time) are assistive and
  always cited/reviewable — never baked into a rule. The symbolic core stays
  auditable.

## The checker, so far

The first static check is built: **`deon-check`**, the leak detector (DESIGN §4,
check 1). It walks the OKF-frontmatter norm schema and flags the mechanical /
judgment seam being crossed silently — a `mechanical` test computing on a
judgment (LEAK-1), an undeclared/uncolored input (LEAK-2), or a
`judgment-aggregation` faked as a formula (LEAK-3).

```sh
nix run . -- examples/            # the seed norms → clean (exit 0)
cargo run -- tests/fixtures/leaky.okf.md   # the red fixture → 3 located leaks (exit 1)
```

It ships with both a green case (the seed norms, authored honestly) and a red
case (a deliberately-leaky fixture), because a checker you've only seen say
"clean" isn't a checker. `nix flake check` builds, lints, and tests it.

## Status

Exploratory. The design rests on two converging paper spikes
([`docs/spikes/`](docs/spikes/)) and the design note
([`docs/DESIGN.md`](docs/DESIGN.md)); [`examples/`](examples/) holds the two
concepts as seed norm files. Grounding-completeness (check 2) and the remaining
DESIGN §4 checks are still to come; no execution engine and no neural components
are built yet — see the design note's Non-goals.

[okf-spec]: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
[pacioli-split]: https://github.com/ojhermann-org/pacioli#why-this-split
[pacioli-seam]: https://github.com/ojhermann-org/pacioli#the-interface-contract-the-crux
