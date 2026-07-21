---
concept: wrong-regime-fixture
title: Deliberately regime-incoherent fixture
# no document-level regime: — so a norm that declares none has no effective regime
norms:
  - id: r2-cross-regime-threshold    # REGIME-2: IFRS-16 threshold in an ASC-840 norm
    subject: thing
    deontic: obligation
    regime: ASC-840
    antecedent:
      is-big:
        mechanical:
          test: "thing.ratio >= threshold"
          threshold: { value: 0.75, regime: IFRS-16, color: mechanical }
    commitment: { flag: true }

  - id: r1-no-regime                 # REGIME-1: no own regime, no document regime
    subject: thing
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: p, color: mechanical, test: thing.flag }
    commitment: { flag: true }
---

# Wrong-regime fixture (test data — not a real norm)

`r2-cross-regime-threshold` pulls a `@IFRS-16` bright-line into a norm scoped to
`ASC-840` (**REGIME-2**); `r1-no-regime` declares no regime and the document
declares none, so it cannot be scoped (**REGIME-1**). Intentionally incoherent —
never an authored norm.
