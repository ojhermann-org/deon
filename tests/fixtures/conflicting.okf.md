---
concept: conflicting-fixture
title: Deliberately conflicting fixture
regime: IFRS-15
norms:
  - id: base-timing
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: transferred, color: mechanical, test: thing.transferred }
    commitment: { timing: over-time }

  - id: c2-judgment-defeat           # CONFLICT-2: collides, bound by a judgment
    subject: thing
    deontic: obligation
    defeats: base-timing
    binds: { predicate: highly-uncertain, color: judgment,
             grounds: { ref: "#uncertain", source: standard-criterion } }
    commitment: { timing: point-in-time }

  - id: c3-mechanical-defeat         # CONFLICT-3: collides, bound mechanically
    subject: thing
    deontic: obligation
    defeats: base-timing
    binds: { predicate: past-cutoff, color: mechanical, test: thing.date > cutoff }
    commitment: { timing: point-in-time }

  - id: c1-dangling-defeat           # CONFLICT-1: defeats a norm that isn't here
    subject: thing
    deontic: obligation
    defeats: no-such-norm
    binds: { predicate: whatever, color: mechanical, test: thing.flag }
    commitment: { timing: point-in-time }

  - id: quiet-disjoint-defeat        # silent: collides on nothing
    subject: thing
    deontic: permission
    defeats: base-timing
    binds: { predicate: elected, color: election,
             grounds: { ref: "#elected", source: entity-election } }
    commitment: { capitalize: false }

  - id: quiet-unconditional-defeat   # silent: collides, but priority is settled
    subject: thing
    deontic: obligation
    defeats: base-timing
    commitment: { timing: point-in-time }
---

# Conflicting fixture (test data — not a real norm)

Four defeat edges against `base-timing`, plus one dangling edge.
`c2-judgment-defeat` and `c3-mechanical-defeat` both collide with it on `timing`;
the first is bound by a judgment hole, so it is **CONFLICT-2**
(`underdetermined(highly-uncertain)`), and the second is bound mechanically, so
it is **CONFLICT-3** (a determinate conflict). `c1-dangling-defeat` names a norm
that does not exist (**CONFLICT-1**).

The last two are deliberately silent, and pin the definition of a conditional
conflict: `quiet-disjoint-defeat` constrains a different field, so there is
nothing to resolve; `quiet-unconditional-defeat` collides but carries no `binds`,
so its priority is settled rather than conditional. Intentionally incoherent —
never an authored norm.
