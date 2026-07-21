---
concept: mixed-forms-fixture
title: Deliberately mixed branch forms — trips SEAM-3
regime: TEST
norms:
  - id: guard-and-cases              # SEAM-3: a stray `antecedent` beside `cases:`
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: in-scope, color: mechanical, test: thing.in-scope }
    cases:
      - when:
          all-of:
            - { predicate: a, color: mechanical, test: thing.a }
        commitment: { classify: a }
      - commitment: { classify: none }

  - id: two-residuals                # SEAM-3: `otherwise` beside a `when`-less case
    subject: thing
    deontic: obligation
    cases:
      - when:
          all-of:
            - { predicate: b, color: mechanical, test: thing.b }
        commitment: { classify: b }
      - commitment: { classify: none }
    otherwise:
      commitment: { classify: other }

  - id: unconditional-and-cases      # SEAM-3: a top-level `commitment` beside `cases:`
    subject: thing
    deontic: obligation
    commitment: { classify: always }
    cases:
      - when:
          all-of:
            - { predicate: c, color: mechanical, test: thing.c }
        commitment: { classify: c }
      - commitment: { classify: none }
---

# Mixed-forms fixture (test data — not a real norm)

Three ways to mix the binary and n-ary branch forms, each undefined in a
different way (DESIGN §3). `guard-and-cases` carries an `antecedent` that no
check reads for branch structure — dead text that still passes LEAK and GROUND,
and that an author would reasonably mistake for a guard. `two-residuals` gives
the norm two mutually exclusive residuals (a `when`-less case *and* an
`otherwise`), so which one takes effect is unstated. `unconditional-and-cases`
commits unconditionally *and* splits into cases, which contradict each other.

Every branch here carries a commitment, so nothing trips SEAM-1 or SEAM-2 — the
malformedness is the *shape*, not a dead end. Intentionally malformed — never an
authored norm.
