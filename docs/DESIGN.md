# deon design note: a colored deontic norm layer for accounting judgment

Status: exploratory design, representational-only. Rests on two paper spikes
([`spikes/revenue-recognition-timing.md`](spikes/revenue-recognition-timing.md),
[`spikes/lease-classification.md`](spikes/lease-classification.md)) and two seed
norm files ([`../examples/`](../examples/)). No execution semantics proposed yet
— see Non-goals.

deon is the judgment-side sibling to
**[Pacioli](https://github.com/ojhermann-org/pacioli)** (the verified Lean
mechanics); its bottom edge _is_ [Pacioli's seam][pacioli-seam], and "the Lean
seam" / "downstream Lean" throughout this note always refer to that library.

## 1. Purpose

A structured, checkable layer _inside_ the judgment side that sits between
**Open Knowledge Format** ([OKF][okf-spec]) prose and the plain-data seam Lean
consumes. It does not replace OKF and does not
touch Lean types. Its value is three things Lean-and-prose cannot give on their
own:

- **consistency** — detect conflicting obligations, coverage gaps, dead rules;
- **traceability** — from a proposed seam input back to the exact judgments it
  rests on;
- **auditable consequence-propagation** — "given these facts, this obligation
  follows, and it hinges on judgments P and Q, ungrounded."

It _assists_ Lean + OKF downstream; it is not a new authority. Lean proves the
mechanics are right given the inputs; deon shows the inputs are justified and
names the judgment chain.

## 2. Placement and hard constraints (bounded on both ends)

- **Top edge grounds in OKF prose + citations.** Open-textured predicates resolve
  there; the prose stays authoritative. If rule and prose disagree, prose wins.
- **Bottom edge grounds in the [Pacioli Lean seam][pacioli-seam].** Every
  well-formed norm terminates in a _commitment about plain data_ the kernel will
  check — the point where, per Pacioli's interface contract, a judgment produces
  plain-data _inputs_ and the Lean kernel consumes them deterministically. A norm
  that never constrains seam data is malformed.
- **Not in the pure core.** This is judgment-side / downstream tooling; it must
  not add an applied surface to the Lean library
  ([Pacioli #41][pacioli-41]) and must not put policy into a Lean type (the seam
  invariant), one level up.
- **The type system encodes the seam.** The mechanical/judgment cut is a checked
  property, exactly as ["policy never leaks into types"][pacioli-seam] is on the
  Lean side.

## 3. Abstract syntax (node types)

A **norm** is:

```
norm        := { id, regime, concept-ref, subject, antecedent, deontic,
                 commitment, otherwise?, defeated-by* }
regime      := standard/jurisdiction scope (e.g. ASC-840, IFRS-16)   # norms are regime-indexed
concept-ref := link to the governing OKF concept
subject     := the record-state the obligation ranges over (a PO, a lease, ...)
deontic     := obligation | permission | prohibition
otherwise   := { commitment }   # residual branch: the commitment taken when the antecedent does NOT hold
```

`regime` and `concept-ref` are **inherited** from the OKF document's frontmatter
(`regime:`, `concept:`) unless a norm sets its own — e.g. the lease document is
`ASC-840` but its `short-term-low-value-election` norm overrides to `IFRS-16`. A
norm without an explicit `otherwise` makes no commitment when its antecedent
fails (that silence is what the coverage check in §4 reasons about).

**Predicates** (the antecedent is built from these) are _colored_:

```
predicate   := mechanical(test, inputs*)                 # decidable from seam data
             | judgment(grounds)                         # open-textured hole; must cite
             | election(grounds: entity-policy)          # a discretionary entity choice
             | violated(norm-id, status: mechanical|judgment)   # compliance as a predicate
```

- `mechanical` may take **inputs that are themselves judgment** (an estimate
  crossing the seam as a number). Its verdict is real but the trace runs through
  those inputs.
- `judgment(grounds)` — `grounds` has a **source type**: `standard-criterion |
  world-fact | legal-fact | entity-election`.

**Connectives and aggregation:**

```
antecedent  := predicate
             | all-of[antecedent...] | any-of[antecedent...] | not(antecedent)   # mechanical glue
             | judgment-aggregation(factors*, grounds)   # weighed factors, NO combination rule
```

The `judgment-aggregation` node is load-bearing: it is how deon refuses to fake
a weighed judgment (e.g. IFRS 15.38 point-in-time indicators) as a formula.
Mechanical connectives may compose over judgment atoms — this is the core
discovery (a standard's _structure_ is mechanical, its _criteria_ judgment).

**Thresholds** are colored artifacts, not inert numbers:

```
threshold   := const(value) @regime        # e.g. 0.75 @ASC-840  (mechanized judgment)
             | judgment(grounds)            # e.g. "major part"   @IAS-17  (not mechanized)
```

**Commitment** (the bottom edge):

```
commitment  := { deontic-target, method?: judgment, estimate-inputs*: judgment,
                 modifies?: <field := capped/adjusted value> }
```

The commitment carries a _method choice + estimate inputs_ (both judgment); the
_arithmetic_ that turns them into per-period/per-account numbers is downstream
Lean's, not the norm's.

**Priority / defeat** is itself colored:

```
defeat      := { defeats: norm-id, binds: predicate, modifies?: <field := adjusted value> }   # binds may be judgment
```

**Concrete rendering (how §3 maps to `examples/`).** §3 is _abstract_ syntax; the
seed norms render it as OKF frontmatter YAML. Each lives in a `<concept>.okf.md`
file — OKF-format markdown: the norm block is the YAML frontmatter, the
authoritative cited prose is the body beneath it. The field names differ from the
abstract grammar in two deliberate ways:

- A commitment's `deontic-target` is written as the **domain field(s) it sets**
  — `classification: finance`, `timing: over-time`, `capitalize: false`,
  `adjustment: prior-period` — rather than the literal key `deontic-target`;
  the structural keys `method`, `estimate-inputs`, and `modifies` keep their
  grammar names where they appear (e.g. `method: retrospective`).
- A `defeat` appears in **either** of two shapes: as a **standalone** node
  (`{ defeats, binds, modifies? }`, e.g. `var-consideration-constraint`
  modifying the defeated norm's commitment), **or** as a **full norm carrying a
  `defeats:` field** with its own `antecedent`/`commitment` (e.g.
  `short-term-low-value-election`). Both are the `defeat` node above; the second
  just co-locates it with a norm that also stands on its own.

## 4. Static checks (representational; no execution)

1. **Leak detection.** No `judgment`/`election` predicate is evaluated as
   `mechanical`; every `mechanical` test's inputs are declared and colored.
   _Implemented_ as the `deon-check` crate (LEAK-1/2/3); run it with
   `nix run . -- examples/`.
2. **Grounding completeness.** Every `judgment` hole has a `grounds` ref that
   resolves to a real OKF concept anchor, with a declared source type.
3. **Coverage.** The antecedent branches partition the subject's relevant states;
   flag implicit gaps (spike 1 found "PO not yet satisfied → recognize nothing"
   was unrepresented). An `otherwise` branch makes the split _syntactically_
   total (antecedent-holds → `commitment`, else → `otherwise.commitment`), but
   coverage still checks that this binary matches the subject's real states — the
   "recognize nothing" third state is a gap an over-time/point-in-time `otherwise`
   hides, not one it closes.
4. **Conditional conflict.** When norm A `defeats` norm B via a judgment `binds`,
   report the conflict as _underdetermined until grounded_, not a static
   contradiction.
5. **Termination-at-seam.** Every norm's obligation reaches a `commitment` about
   plain data; flag any that don't.
6. **Regime hygiene.** A norm applies only within its `regime`; flag facts
   evaluated against a norm whose regime doesn't apply (e.g. lessee
   classification under IFRS-16 — the norm doesn't exist there).

## 5. Signature capability (the reason to build it)

Run the commitment backward: **given a proposed set of seam inputs, emit the
trace of judgments they depend on, and flag any unresolved or conflicting ones.**
Verdicts are three-valued: `complied | violated | underdetermined(P...)`. Even a
purely mechanical bright-line test traces _through_ its estimate inputs to the
judgments beneath (spike 2, N2).

## 6. Where "neuro" attaches (kept out of the symbolic core)

- **Extraction** (prose → candidate norms): assistive, human-ratified. Low risk.
- **Grounding** the open-textured holes at evaluation time ("is control
  transferred?", "is it material?"): higher stakes — must surface _which_ OKF
  concept + citation it grounded against, never decide invisibly.
- **Conflict-resolution suggestions**: detection is symbolic; only suggestions
  are neural.

The symbolic layer stays auditable; neural contributions are always explicit,
cited, and reviewable — never baked into a rule.

## 7. Evidence (two converging spikes)

- **Rev-rec timing** (open-textured): established the core — mechanical
  connectives over judgment atoms; judgment-aggregation node; contrary-to-duty
  for free; colored-judgment priority; method+estimate commitments; a real
  coverage gap found statically.
- **Lease classification** (bright-line, adversarial contrast): four of five
  features held verbatim; growth was one new feature — regime-scoped norms +
  colored threshold constants — which is exactly the axis a bright-line concept
  probes. The space is bracketing, not exploding.

**Spec-able core:** (1) colored predicates — mechanical / judgment / election;
(2) grounding-source taxonomy; (3) mechanical connectives over judgment atoms;
(4) judgment-aggregation nodes; (5) regime-scoped norms + colored thresholds;
(6) three-valued compliance; (7) colored-judgment priority; (8) commitments =
method + estimate bundles feeding downstream Lean.

## 8. Open questions

- **Regime as a first-class dimension.** Norms differ by ASC 840 / IAS 17 / IFRS
  16, sometimes existing under one and not another. How much regime machinery is
  warranted before it becomes its own project?
- **Grounding-source taxonomy** — is `{standard-criterion, world-fact,
  legal-fact, entity-election}` complete, and does each source want different
  handling (legal facts may reference law, not accounting literature)?
- **Priority when the ordering itself conflicts** — colored-judgment priority
  handles "does A defeat B," but not yet "two defeats disagree."
- **Overlap with existing standards** (SBVR modalities, LegalRuleML, Governatori's
  defeasible deontic logic) vs. what is genuinely ours (the two bounded edges).
  Adopt where we can; the seam-anchoring is the novel part.

## 9. Non-goals (deliberately deferred)

- **No execution semantics yet.** Static analysis over a normal form is the first
  deliverable; a facts→verdict engine is a much bigger commitment, earned later.
- **No concrete/custom syntax yet.** Begin as a disciplined schema inside OKF
  frontmatter so every norm sits beside its cited prose.
- **No neural components built.** Section 6 marks where they attach; none are
  proposed for the first cut.
- **Nothing lands in the pure Lean core.**

[okf-spec]: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
[pacioli-seam]: https://github.com/ojhermann-org/pacioli#the-interface-contract-the-crux
[pacioli-41]: https://github.com/ojhermann-org/pacioli/issues/41
