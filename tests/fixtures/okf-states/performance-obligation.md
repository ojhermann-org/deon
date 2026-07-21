---
concept: revenue-recognition-timing
subjects:
  performance-obligation:
    states:
      - { id: not-yet-satisfied,
          grounds: { ref: "#ifrs15-31", source: standard-criterion } }
      - { id: satisfied-over-time,
          grounds: { ref: "#ifrs15-35", source: standard-criterion } }
      - { id: satisfied-at-a-point,
          grounds: { ref: "#ifrs15-38", source: standard-criterion } }
  thing:
    states:
      - { id: a, grounds: { ref: "#thing-a", source: standard-criterion } }
      - { id: b, grounds: { ref: "#thing-b", source: standard-criterion } }
---

# Performance obligation — state space (fixture)

A stand-in for the OKF concept prose that would declare what states IFRS 15
recognizes for a performance obligation. Which states a subject *has* is a
judgment about the standard, so it is norm content and lives here beside the
prose that grounds it — not in the checker.

The third state is the one spike 1 found missing from the seed norm (F5):

## Control has not yet transferred {#ifrs15-31}

Revenue is recognized when (or as) a performance obligation is satisfied. Until
then the obligation is in neither the over-time nor the point-in-time state, and
the entity recognizes nothing — the state an over-time/point-in-time binary
hides rather than closes.

## Satisfied over time {#ifrs15-35}

Prose for the over-time criteria.

## Satisfied at a point in time {#ifrs15-38}

Prose for the point-in-time indicators.

## Thing, state a {#thing-a}

Prose for the generic `thing` subject the red fixtures are checked against.

## Thing, state b {#thing-b}

Prose for the other state of `thing`.
