---
concept: lease-classification
title: Lease classification — finance vs operating (lessee)
regime: ASC-840
sources:
  - ASC 840        # bright-line classification (superseded)
  - IAS 17         # substantially-all risks and rewards (superseded)
  - IFRS 16        # lessee capitalizes all leases
norms:
  - id: lease-classification
    subject: lease
    deontic: obligation
    antecedent:
      is-finance-lease:                # mechanical any-of over mixed atoms
        any-of:
          - { predicate: ownership-transfers, color: mechanical,
              test: lease.transfers-ownership }
          - { predicate: purchase-option-certain, color: judgment,
              grounds: { ref: "#option", source: standard-criterion } }
          - term-major-part:
              mechanical:
                test: "lease.term / economic-life >= threshold"
                threshold: { value: 0.75, regime: ASC-840, color: mechanical }
                inputs: { economic-life: { color: judgment, source: world-fact } }
          - pv-substantially-all:
              mechanical:
                test: "pv(lease.payments, rate) / fair-value >= threshold"
                threshold: { value: 0.90, regime: ASC-840, color: mechanical }
                inputs:
                  fair-value: { color: judgment, source: world-fact }
                  rate:       { color: judgment, source: world-fact }
          - { predicate: specialized-no-alt-use, color: judgment,
              grounds: { ref: "#specialized", source: standard-criterion } }
    commitment: { classification: finance }
    otherwise:
      commitment: { classification: operating }
    note: measurement (ROU asset + lease liability) is downstream Lean

  - id: short-term-low-value-election  # an election defeats the default
    subject: lease                     # ranges over the same record it tests
    regime: IFRS-16
    deontic: permission
    defeats: lease-classification
    antecedent:
      all-of:
        - { predicate: short-term, color: mechanical,
            test: "lease.term <= threshold",
            threshold: { value: "12mo", regime: IFRS-16, color: mechanical } }
        # Colored `judgment` but sourced `entity-election`: IFRS 16's exemption
        # is an entity election whose threshold ("low value") is open-textured.
        # Whether that is one node or two — the election, plus a judgment inside
        # it — is a question about the norm, not the language (DESIGN §3).
        - { predicate: low-value, color: judgment,
            grounds: { ref: "#low-value", source: entity-election } }
    commitment: { capitalize: false }

regime-note: >
  Under IFRS-16 the lessee classification norm above does not exist — all lessee
  leases capitalize. A norm's existence is regime-relative (spike 2, N1).
---

# Lease classification (lessee)

The cited judgment prose lives here, beside the norm block. This stub stands in
for the OKF concept that the `grounds:` refs point into (`#option`,
`#specialized`, `#low-value`). This seed passes the always-on checks clean —
leak detection (check 1) and structural grounding (check 2, GROUND-1/2): every
criterion is cited and typed. Only anchor _resolution_ (GROUND-3, under `--okf`)
is expected to flag, because that OKF bundle does not exist yet, so the refs
dangle until it lands.

Note the two regime-scoped thresholds (`0.75 @ASC-840`, `0.90 @ASC-840`,
`12mo @IFRS-16`): each is a **colored artifact**, not an inert number — the
mechanization of a judgment that a different regime leaves open ("major part",
"substantially all").
