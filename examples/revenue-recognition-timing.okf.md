---
concept: revenue-recognition-timing
title: Revenue recognition — timing (when control transfers)
regime: IFRS-15
sources:
  - IFRS 15.31–38  # satisfaction of performance obligations
  - IFRS 15.39–45  # measuring progress
  - IFRS 15.56–58  # constraining variable consideration
  - IAS 8          # errors vs changes in estimate
norms:
  - id: rev-rec-timing
    subject: performance-obligation
    deontic: obligation
    antecedent:                        # mechanical any-of over judgment atoms
      over-time:
        any-of:
          - { predicate: simultaneous-receipt-consumption, color: judgment,
              grounds: { ref: "#ifrs15-35a", source: standard-criterion } }
          - { predicate: customer-controlled-asset, color: judgment,
              grounds: { ref: "#ifrs15-35b", source: standard-criterion } }
          - { predicate: no-alt-use-and-enforceable-payment, color: judgment,
              grounds: { ref: "#ifrs15-35c", source: legal-fact } }
    commitment:
      timing: over-time
      measure: { color: judgment, grounds: { ref: "#ifrs15-39", source: standard-criterion } }
      note: arithmetic (measure → per-period amounts) is downstream Lean
    otherwise:                         # residual branch
      commitment:
        timing: point-in-time
        point:
          judgment-aggregation:        # weighed indicators, NO combination rule
            factors: [right-to-payment, legal-title, possession, risks-rewards, acceptance]
            grounds: { ref: "#ifrs15-38", source: standard-criterion }

  - id: var-consideration-constraint
    concept-ref: variable-consideration
    defeats: rev-rec-timing            # priority is itself a colored-judgment hole
    binds: { predicate: highly-probable-no-reversal, color: judgment,
             grounds: { ref: "#ifrs15-56", source: standard-criterion } }
    modifies: { commitment.amount: cap-to-no-significant-reversal }

  - id: rev-rec-restatement            # contrary-to-duty
    deontic: obligation
    antecedent:
      all-of:
        - { predicate: violated, norm: rev-rec-timing, color: judgment,
            grounds: { ref: "#ias8-error-vs-estimate", source: standard-criterion } }
        - { predicate: material, color: judgment,
            grounds: { ref: "materiality", source: standard-criterion } }
    commitment: { adjustment: prior-period, method: retrospective }

coverage-note: >
  "Performance obligation not yet satisfied → recognize nothing" is a third state
  not represented above; the checker flags it as a coverage gap (spike 1, F5).
---

# Revenue recognition — timing

The cited judgment prose lives here, beside the norm block, as an OKF concept.
This stub is a placeholder for that prose; in a real OKF bundle it would carry the
full reasoning and citations that the `grounds:` refs above point into
(`#ifrs15-35a`, `#ifrs15-38`, …). This seed passes the always-on checks clean —
leak detection (check 1) and structural grounding (check 2, GROUND-1/2): every
criterion is cited and typed. Only anchor _resolution_ (GROUND-3, under `--okf`)
is expected to flag, because that OKF bundle does not exist yet, so the refs
dangle until it lands.

This file demonstrates the deon convention: **the formal norm is frontmatter; the
authoritative judgment is the prose beneath it.** If they ever disagree, the prose
wins and the norm is the bug.
