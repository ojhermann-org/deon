---
concept: leaky-fixture
title: Deliberately leaky fixture — trips each leak rule once
regime: TEST
norms:
  - id: leak1-judgment-computed        # LEAK-1: mechanical test embeds `is-material`
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: is-material, color: judgment,
            grounds: { ref: "#materiality", source: standard-criterion } }
        - predicate: over-threshold
          color: mechanical
          test: "thing.ratio >= threshold and is-material"
          threshold: { value: 0.05, regime: TEST, color: mechanical }
    commitment: { flag: true }

  - id: leak2-undeclared-input         # LEAK-2: `benchmark` is neither field nor input
    subject: thing
    deontic: obligation
    antecedent:
      is-big:
        mechanical:
          test: "thing.size / benchmark >= threshold"
          threshold: { value: 100, regime: TEST, color: mechanical }
    commitment: { flag: true }

  - id: leak3-faked-aggregation        # LEAK-3: aggregation carries a formula
    subject: thing
    deontic: obligation
    antecedent:
      weigh:
        judgment-aggregation:
          factors: [a, b, c]
          grounds: { ref: "#weighing", source: standard-criterion }
          test: "0.4*a + 0.4*b + 0.2*c >= threshold"
    commitment: { flag: true }
---

# Leaky fixture (test data — not a real norm)

This file exists only to prove the checker's teeth: it trips **LEAK-1**,
**LEAK-2**, and **LEAK-3** exactly once each. It is intentionally dishonest and
must never be treated as an authored norm.
