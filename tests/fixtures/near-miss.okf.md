---
concept: near-miss-fixture
title: The near-miss forms — one line away from silencing each rule
regime: TEST
norms:
  - id: n1-dotted-judgment          # LEAK-1: a field access on a judgment name
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: is-material, color: judgment,
            grounds: { ref: "#m", source: standard-criterion } }
        - predicate: leaky
          color: mechanical
          test: "thing.ratio >= threshold and is-material.value"
          threshold: { value: 0.05, regime: TEST, color: mechanical }
    commitment: { flag: true }

  - id: n2-foreign-record           # LEAK-2: a dotted name rooted elsewhere
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: elsewhere, color: mechanical,
            test: contract.control_transferred }
    commitment: { flag: true }

  - id: n3-uncolored-input          # LEAK-2: declared, but says nothing
    subject: thing
    deontic: obligation
    antecedent:
      is-big:
        mechanical:
          test: "thing.size / benchmark >= threshold"
          threshold: { value: 100, regime: TEST, color: mechanical }
          inputs: { benchmark: {} }
    commitment: { flag: true }

  - id: n4-uncited-aggregation      # LEAK-3 + GROUND-1: no `grounds` at all
    subject: thing
    deontic: obligation
    antecedent:
      weigh:
        judgment-aggregation:
          factors: [a, b, c]
          test: "0.4*a + 0.4*b + 0.2*c >= threshold"
    commitment: { flag: true }
---

# Near-miss fixture (test data — not a real norm)

Every other red fixture in this repo demonstrates the **canonical** form of a
defect. This one demonstrates the *near-miss* forms — the small edits that used
to make a rule stop applying instead of making it fire. Each norm here was
silent before the recognition fix, and `tests/fixtures/leaky.okf.md` was one
deleted line (`grounds:`) or one added line (`inputs: { benchmark: {} }`) away
from being entirely green.

- `n1-dotted-judgment` appends `.value` to a name declared `judgment` in this
  same file. Waving every dotted token through as "seam data" let any judgment
  be laundered by giving it a field.
- `n2-foreign-record` computes on `contract.…` in a norm that ranges over
  `thing` — a record it never declares. LEAK-2's message always claimed to
  require "a subject field or a declared colored input"; now it checks.
- `n3-uncolored-input` declares `benchmark` and says nothing about it. An entry
  that names no color is not a declaration.
- `n4-uncited-aggregation` omits `grounds`, which used to stop it from being
  recognized as an aggregation at all — invisible to GROUND-1, the rule that
  exists to demand the citation, *and* to LEAK-3, despite the formula.

The principle each of these enforces: **recognize a node by the weakest signal
that it was intended, then check the rest.** What a rule checks for must never
also be what makes the rule apply, or the check gets weaker exactly as the input
gets more dishonest. Intentionally malformed — never an authored norm.
