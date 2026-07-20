# Spike 1: revenue-recognition timing as a colored deontic norm

Concept: IFRS 15 / ASC 606 step 5 — recognize revenue when (or as) control of a
performance obligation (PO) transfers to the customer. Chosen because it is
defeasible, has a contrary-to-duty tail (restatement), and its core predicate
(control transfer) is open-textured.

## The standard's structure (grounding)

- A PO is satisfied **over time** if ANY of three criteria hold (IFRS 15.35):
  (a) customer simultaneously receives and consumes the benefits;
  (b) performance creates/enhances an asset the customer controls as it is made;
  (c) no alternative use to the entity AND an enforceable right to payment for
  performance completed to date.
- Otherwise the PO is satisfied at a **point in time**, determined by weighing
  indicators (IFRS 15.38): present right to payment, legal title, physical
  possession, risks/rewards, customer acceptance.
- Over-time recognition measures progress by an output or input method (IFRS
  15.39–45). Variable consideration is capped: recognize only to the extent a
  significant reversal is highly probable not to occur (IFRS 15.56).
- Mis-timing correction: an **error** → retrospective restatement (IAS 8), gated
  on materiality; a **change in estimate** → prospective, and is not a violation.

## Predicate inventory (colored)

| Predicate | Color | Grounds in | Note |
|---|---|---|---|
| `simultaneous-receipt-consumption` | judgment | 15.35(a) | open-textured "benefits" |
| `customer-controlled-asset` | judgment | 15.35(b) | leans on control definition |
| `no-alt-use-and-enforceable-payment` | judgment | 15.35(c) | judgment about a legal fact |
| `over-time` | **mechanical** | 15.35 | `= (a) ∨ (b) ∨ (c)` — fixed disjunction |
| `point-in-time-point` | judgment | 15.38 | weighed indicators, NO formula |
| `progress-measure` | judgment | 15.39 | output vs input method choice |
| `highly-probable-no-reversal` | judgment | 15.56 | gates the constraint's priority |
| `violated(rev-rec-timing)` | judgment | IAS 8 | error vs change-in-estimate |
| `material` | judgment | materiality | gates restatement |

## Findings

**Headline (encouraging):** the standard's _structure_ is mechanical; its
_criteria_ are judgment. Coloring is compositional — mechanical connectives over
judgment atoms. The seam runs _through_ the rule, not around it.

**F1 — two shapes of "when".** Over-time is a mechanical disjunction;
point-in-time is an irreducible judgment aggregation with no combination rule.
The language needs a first-class "weighed factors, no formula" node or it will
tempt us to fake indicators as a formula = false precision.

**F2 — compliance status can be judgment.** `violated(rev-rec-timing)` isn't
mechanically decidable (error vs change-in-estimate is judgment). Contrary-to-duty
holds structurally, but the violation predicate it conditions on may itself be a
colored hole. The engine must be able to return "violation-status
underdetermined."

**F3 — priority is a colored-judgment artifact.** The variable-consideration cap
defeats the timing norm only if "highly probable no significant reversal." The
superiority relation is data-with-holes, not a fixed order.

**F4 — the bottom edge is sometimes a schedule-generator.** Over-time `commits`
is a method + estimate inputs (total expected cost, standalone selling prices);
the per-period arithmetic is Lean's. Estimates are judgment crossing the seam as
plain numbers — consistent with the contract.

**F5 (coverage win):** the static checker flags a real gap — neither branch
represents "PO not yet satisfied → recognize nothing," an implicit third state.

**Verdict:** the approach holds. The coloring stayed honest — nowhere did a
judgment have to be frozen into a mechanical predicate; where something resisted
(point-in-time), the resistance demanded a new node type, not a fudge.
