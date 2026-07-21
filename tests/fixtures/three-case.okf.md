---
concept: three-case-fixture
title: N-ary `cases:` fixture — the third state the binary form cannot express
regime: IFRS-15
norms:
  - id: full-cases                   # well-formed: every case commits
    subject: performance-obligation
    deontic: obligation
    cases:
      - when:
          over-time:
            any-of:
              - { predicate: consumed-as-delivered, color: judgment,
                  grounds: { ref: "#ifrs15-35a", source: standard-criterion } }
        commitment: { timing: over-time }
      - when:
          all-of:
            - { predicate: satisfied, color: mechanical, test: po.satisfied }
        commitment: { timing: point-in-time }
      - commitment: { timing: none }  # residual: the `when`-less third state

  - id: dead-end-case                # SEAM-2: the middle case commits nothing
    subject: performance-obligation
    deontic: obligation
    cases:
      - when:
          all-of:
            - { predicate: a, color: mechanical, test: po.a }
        commitment: { timing: over-time }
      - when:
          all-of:
            - { predicate: b, color: mechanical, test: po.b }
        note: no commitment on this case
      - commitment: { timing: none }

  - id: defeats-a-middle-case        # CONFLICT-2 against `full-cases`' middle case
    subject: performance-obligation
    deontic: obligation
    defeats: full-cases
    binds: { predicate: control-retained, color: judgment,
             grounds: { ref: "#ifrs15-38", source: standard-criterion } }
    commitment: { timing: over-time }
---

# Three-case fixture (test data — not a real norm)

`full-cases` is the shape the binary form cannot express: over-time,
point-in-time, **and** the `when`-less residual "recognize nothing" third state
(spike 1, F5). It terminates cleanly — every case carries its own commitment.

`dead-end-case` proves termination reaches *inside* the n-ary form: its middle
case commits nothing, so it trips **SEAM-2** at `cases[1]` rather than passing
unseen. `defeats-a-middle-case` proves conditional conflict does too — it
collides with `full-cases` only on that norm's *middle* case
(`timing: point-in-time` vs its own `over-time`), and its judgment `binds` makes
that **CONFLICT-2**, not a static contradiction.

Intentionally part-malformed — never an authored norm.
