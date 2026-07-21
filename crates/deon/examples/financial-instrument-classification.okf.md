---
concept: financial-instrument-classification
title: Financial instrument — liability vs equity (issuer)
regime: IAS-32
sources:
  - IAS 32.11        # definitions: financial liability, equity instrument
  - IAS 32.15–16     # substance over form; the two equity conditions
  - IAS 32.16A–16D   # puttable instruments exception
  - IAS 32.25        # contingent settlement provisions
  - IAS 32.28–32     # compound instruments
norms:
  - id: instrument-classification
    subject: financial-instrument
    deontic: obligation
    antecedent:
      is-financial-liability:          # mechanical any-of over judgment atoms
        any-of:
          - { predicate: contractual-obligation-to-deliver-cash, color: judgment,
              grounds: { ref: "#ias32-11-liability", source: standard-criterion } }
          - { predicate: obligation-to-exchange-on-unfavourable-terms, color: judgment,
              grounds: { ref: "#ias32-11-unfavourable", source: standard-criterion } }
          - settles-in-variable-own-equity:
              mechanical:
                test: "financial-instrument.settles-in-own-equity and not fixed-for-fixed"
                inputs:
                  # "fixed amount of cash for a fixed number of own equity
                  # instruments" — a legal reading of the contract terms, which
                  # crosses the seam as a value, not as a computation.
                  fixed-for-fixed: { color: judgment, source: legal-fact }
    commitment: { classification: liability }
    otherwise:
      commitment: { classification: equity }

  - id: contingent-settlement          # IAS 32.25
    subject: financial-instrument
    deontic: obligation
    defeats: instrument-classification
    binds: { predicate: contingency-is-genuine, color: judgment,
             grounds: { ref: "#ias32-25", source: standard-criterion } }
    commitment: { classification: liability }

  - id: puttable-exception             # IAS 32.16A–16D
    subject: financial-instrument
    deontic: permission
    defeats: instrument-classification
    binds: { predicate: meets-puttable-conditions, color: judgment,
             grounds: { ref: "#ias32-16a", source: standard-criterion } }
    commitment: { classification: equity }

  - id: compound-instrument            # IAS 32.28–32 — see the note below
    subject: financial-instrument
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: contains-both-liability-and-equity-components, color: judgment,
            grounds: { ref: "#ias32-28", source: standard-criterion } }
    commitment:
      classification: compound
      method: { value: residual-allocation, color: mechanical }

seam-note: >
  The first three norms terminate exactly on Pacioli's seam: `classification:
  liability | equity` is the argument its injected `classify : α → AccountClass`
  wants (`AccountClass.claim .liability` / `.claim .equity`). `compound-instrument`
  does not — see the body.
---

# Financial instrument — liability vs equity (issuer)

A third seed, encoded to **test the language** rather than to grow a catalog of
norms (CLAUDE.md: deon owns the language, not the norms). It was chosen because
its commitment lands on the one place Pacioli already expects a value from the
judgment half — `classify : α → AccountClass` — so it probes the seam contract
that the two repos assert but nothing has exercised.

This stub stands in for the OKF concept the `grounds:` refs point into. The
encoding is a test artifact, not an authority on IAS 32.
