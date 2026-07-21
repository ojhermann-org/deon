---
concept: bad-state-declarations
subjects:
  thing:
    states:
      - { id: grounded,
          grounds: { ref: "#real", source: standard-criterion } }
      - { id: uncited }                                    # GROUND-1: no ref
      - { id: mistyped,
          grounds: { ref: "#real", source: vibes } }        # GROUND-2: bad source
      - { id: dangling,
          grounds: { ref: "#nowhere", source: standard-criterion } }  # GROUND-3
      - { grounds: { ref: "#real", source: standard-criterion } }     # COVER-3: no id
---

# Bad state declarations (fixture)

A bundle that fails to ground its own claims about what states a subject has.
Only `grounded` is well-formed.

The last entry is the quiet one: with no `id` it declares nothing, so it drops
out of the state space entirely — and a state that is not in the space is a
state coverage stops looking for. That is a coverage gap created by a typo, with
no finding anywhere until COVER-3.

## A real anchor {#real}

Prose for the declarations that resolve.
