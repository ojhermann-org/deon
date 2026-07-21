---
concept: unterminated-fixture
title: Deliberately unterminated fixture — trips the seam rules
regime: TEST
norms:
  - id: s1-no-commitment             # SEAM-1: no commitment and no modifies
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: p, color: mechanical, test: thing.flag }

  - id: s2-empty-commitment          # SEAM-2: commitment present but empty
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: q, color: mechanical, test: thing.flag }
    commitment: {}

  - id: s2-otherwise-dead-end        # SEAM-2: residual branch carries no commitment
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: r, color: mechanical, test: thing.flag }
    commitment: { classify: a }
    otherwise:
      note: no commitment on this residual branch
---

# Unterminated fixture (test data — not a real norm)

Trips **SEAM-1** (a norm whose obligation reaches no commitment) and **SEAM-2**
twice (an empty commitment, and an `otherwise` residual branch that carries no
commitment). Intentionally malformed — never an authored norm.
