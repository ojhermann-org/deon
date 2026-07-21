---
concept: claim-shapes-fixture
title: Claims written in shapes the checker used to skip
regime: TEST
norms:
  - id: base
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: p, color: mechanical, test: thing.p }
    covers: [a, b]                   # a branch may cover several states
    commitment: { timing: over-time }

  - id: list-defeater                # CONFLICT-2 + CONFLICT-1 from one list
    subject: thing
    deontic: obligation
    defeats: [base, no-such-norm]
    binds: { predicate: uncertain, color: judgment,
             grounds: { ref: "#u", source: standard-criterion } }
    commitment: { timing: point-in-time }

  - id: uncolored-binds              # CONFLICT-4: priority that names no color
    subject: thing
    deontic: obligation
    defeats: base
    binds: bare-predicate-name
    commitment: { timing: point-in-time }

  - id: malformed-claim              # COVER-2: a `covers:` that names no state
    subject: thing
    deontic: obligation
    covers: { state: a }
    antecedent:
      all-of:
        - { predicate: q, color: mechanical, test: thing.q }
    commitment: { timing: point-in-time }
---

# Claim-shapes fixture (test data — not a real norm)

The companion to `near-miss.okf.md`, for the two fields that name *other things*
— `covers:` names a state, `defeats:` names a norm. Both used to be read with a
string accessor, so any other shape yielded nothing and the norm was silently
treated as making no claim at all. Skipping an unreadable claim is the worst
available behaviour: it turns a typo into an opt-out of the very check that
would have caught it.

Two shapes are now **accepted**, because they are meaningful rather than wrong:
`base` covers two states in one branch, and `list-defeater` defeats two norms at
once — which is why that one list yields both a CONFLICT-2 against `base` and a
CONFLICT-1 for the target that does not exist.

Two are **reported**: `uncolored-binds` binds priority to a predicate that names
no color (CONFLICT-4 — the abstract grammar's own bare form, which cannot carry
one), and `malformed-claim` writes a `covers:` that names no state (COVER-2).

Intentionally part-malformed — never an authored norm.
