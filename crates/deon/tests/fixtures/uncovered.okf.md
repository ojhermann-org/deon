---
concept: uncovered-fixture
title: Deliberately incomplete coverage
regime: TEST
norms:
  - id: full-cover                   # green: claims every declared state
    subject: thing
    deontic: obligation
    cases:
      - when:
          all-of:
            - { predicate: p, color: mechanical, test: thing.p }
        covers: a
        commitment: { classify: a }
      - covers: b
        commitment: { classify: b }

  - id: c1-gap                       # COVER-1: claims `a`, never claims `b`
    subject: thing
    deontic: obligation
    covers: a
    antecedent:
      all-of:
        - { predicate: q, color: mechanical, test: thing.q }
    commitment: { classify: a }

  - id: c2-undeclared                # COVER-2: `z` is not a declared state
    subject: thing
    deontic: obligation
    cases:
      - when:
          all-of:
            - { predicate: r, color: mechanical, test: thing.r }
        covers: a
        commitment: { classify: a }
      - when:
          all-of:
            - { predicate: s, color: mechanical, test: thing.s }
        covers: b
        commitment: { classify: b }
      - covers: z
        commitment: { classify: z }

  - id: no-claim                     # skipped: makes no coverage claim at all
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: t, color: mechanical, test: thing.t }
    commitment: { classify: a }

  - id: unknown-subject              # COVER-2: the bundle declares no states here
    subject: widget
    deontic: obligation
    covers: x
    antecedent:
      all-of:
        - { predicate: u, color: mechanical, test: widget.u }
    commitment: { classify: x }
---

# Uncovered fixture (test data — not a real norm)

Checked against a bundle declaring `thing` with states `a` and `b`.

`full-cover` claims both and is clean. `c1-gap` claims `a` and never claims `b`,
so **COVER-1** names the state it missed. `c2-undeclared` claims a state `z` the
subject does not declare (**COVER-2**), while still covering `a` and `b` — so the
undeclared claim is the only finding. `unknown-subject` claims a state for a
subject the bundle says nothing about, which is **COVER-2** with a different
reason.

`no-claim` is the load-bearing one: it makes no coverage claim, so coverage says
nothing about it even though its single branch plainly does not partition
`thing`'s states. Coverage is opt-in per norm — a norm that claims one state
must claim them all, but a norm that claims none is not guessed at.
