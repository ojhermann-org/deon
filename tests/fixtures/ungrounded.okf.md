---
concept: ungrounded-fixture
title: Deliberately ungrounded fixture — trips each grounding rule
regime: TEST
norms:
  - id: g0-resolves                    # well-formed: cited + typed + resolves
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - predicate: cited-and-real
          color: judgment
          grounds: { ref: "#real-anchor", source: standard-criterion }
    commitment: { flag: true }

  - id: g1-missing-citation            # GROUND-1: judgment criterion, no grounds
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - predicate: open-textured
          color: judgment
    commitment: { flag: true }

  - id: g2-invalid-source              # GROUND-2: source not one of the four
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - predicate: mistyped
          color: judgment
          grounds: { ref: "#somewhere", source: vibes }
    commitment: { flag: true }

  - id: g2-election-cites-standard     # GROUND-2: an election grounded in the standard
    subject: thing
    deontic: permission
    antecedent:
      all-of:
        - predicate: not-really-a-choice
          color: election
          grounds: { ref: "#real-anchor", source: standard-criterion }
    commitment: { flag: true }

  - id: g3-dangling-anchor             # GROUND-3 (under --okf): ref does not resolve
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - predicate: cited-but-missing
          color: judgment
          grounds: { ref: "#does-not-exist", source: standard-criterion }
    commitment: { flag: true }
---

# Ungrounded fixture (test data — not a real norm)

Trips **GROUND-1** (missing citation) and **GROUND-2** twice under the always-on
checks — once for a source outside the four types, once for an `election` that
grounds in the standard's prose rather than in entity policy — and **GROUND-3**
(dangling anchor) when run with `--okf tests/fixtures/okf`.

That second GROUND-2 is what makes `election` a real color rather than a synonym
for `judgment`: DESIGN §3 types it `election(grounds: entity-policy)`, so a
choice the *standard* makes for you is a judgment, however it is labelled.

`g0-resolves` is the well-formed control: cited, typed, and resolvable.
Intentionally dishonest — never an authored norm.
