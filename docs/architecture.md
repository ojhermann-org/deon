<!-- markdownlint-disable MD012 MD013 MD022 MD025 MD032 MD049 -->
# Architecture

How the pieces fit: `okf-tools` (the `okf-graph` and `okf-normative` crates),
`pacioli`, the Knowledge Bundles, and the applied accounting agent.

**Status: working draft.** This is the record of a design conversation, not a
settled specification — it is expected to change as implementation surfaces
issues. The first section is the owner's and is marked "do not delete"; agents
should append below the rule rather than editing it. Markdown linting is relaxed for
this file so the owner's text is preserved exactly as written — reflowing or
restructuring it to satisfy a linter would be editing authored text.

---

# Otto thoughts (do not delete)

## okf-graph: validates the topology an OKF Knowledge Bundle intended to be a knowledge graph
- Generates an in-memory graph with nodes and edges representing [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) Concepts and Links, respectively (each Link can have more than one meaning/label)
- Standard graph algorithms used to identify graph properties and compare against assumptions per use case e.g. should be directional, no cycles, etc
- Validates each Concept in the Knowledge Bundle is a valid format; optionally check for consistency in FrontMatter, Body, and Citation (possibly others, but these will likely matter most for downstream uses)
- favour [absolute cross-linking](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#5-cross-linking)
- [Citations](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#8-citations) must link into a `/references` subdirectory that mirriors external material as first-class OKF concepts; okf-graph will generate this if it doesn't already exist; this will make the full graph clearer and reduce that chance of "hidden" edges e.g. two Citations reference the same URL, but without the `/references` directory, that would be much more expensive to identify
- [log files](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#7-log-files-optional) will be used to record changes


## okf-normative: semantic interpretation of the normative properties of an okf-graph
- input is an okf-graph; output is an enhanced okf-graph; enhanced okf-graph + original OKF Knowledge Bundle can genrated a new enhanced OKF Knowledge Bundle
- normative terminology, taxonomy, and concepts will be captured in Frontmatter; existing Frontmatter content can be used (e.g. Type, tags, etc), but is considered immutable; new key/value pairs will be generated
- [log files](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#7-log-files-optional) will be used to record changes, both to Concepts and the Knowledge Bundle (or relevant sub Knowledge Bundle)
- how we mange the normative asepct of this is tbd, but will be done incrementally with a focus on rigorous thought and empirical validation for each change

## pacioli: the mechanics of accounting
- The mechanics — the deterministic, total parts of accounting, to be codified in Lean 4 so that illegal states are unrepresentable and the invariants that matter are machine-checked  — remains in the repo
- The judgment — the contextual decisions (policy, jurisdiction, timing, materiality, classification), to be curated in the Open Knowledge Format (OKF) so that humans and AI agents can share the same auditable reasoning  — leaves the repo
- pacioli will eventually be exposed via and MCP (and possibly other means) so it can easily and safely be used by agents

## various Knowledge Bundles
- these will be created using okf-graph and, sometimes, okf-normative
- examples include GAAP, IFRS, and other domains with normative content

## tbd titled accounting agent
- it will use one or more Knowledge Bundles generated using okf-graph and okf-normative
- it will use pacioli, likely via an MCP
- the Knowledge Bundles may be seeded or being empty, but okf-graph and okf-normative will also be used by the agent (probably via an MCP) to reliably update and maintain the Knowledge Bundles used in the project

---

# Appended from session discussion (2026-07-21)

Points agreed or raised in conversation, kept separate from the spine above so
the original stays intact. Nothing here is settled beyond what was explicitly
agreed; items marked **open** are parked deliberately.

## Repo and crate structure (agreed)

- The `deon` repo becomes **`okf-tools`**, holding three crates.
- **`okf-graph`** and **`okf-normative`** are new, built fresh. Neither is
  accounting-specific.
- **`deon` stays as a crate**: effectively archived, kept as reference.
- `okf-normative` depends on `okf-graph`, **never the reverse**. Each publishes
  independently, so a consumer can take one without the other.
- One repo rather than two because the interface between the crates will churn
  early, and cross-repo churn is the expensive kind — a version bump plus a
  lockstep release for every interface tweak. Splitting later is cheap;
  unsplitting is not.
- **The boundary must be deliberate**, since inside a workspace it costs nothing
  to violate: no reaching around through dev-dependencies, and `okf-graph`'s
  tests must not mention a color, a norm, or anything normative. A falsifiable
  version of the rule: if its test suite cannot be read by someone who has never
  heard of deontic logic, the boundary is not real.

### On archiving deon

- Keep it a **live workspace member** with `publish = false`, not an excluded
  directory. Its fixtures and 27 tests are a behavioural specification worth
  running as a baseline while the new crates are built. An excluded crate rots
  and stops compiling; a green one stays useful. Exclude later if it drags.
- **Its docs are not archived with it.** `DESIGN.md`, the two spikes, and the IAS
  32 probe are the intellectual foundation `okf-normative` inherits — the seam,
  the coloring model, N1 (whether something is mechanical is a standard-setter's
  policy choice), F5, the grounding discipline. Only the *implementation* is
  superseded. Keep `docs/` at the repo root with a note separating what carries
  forward from what is deon-specific (rule codes, module layout).

## okf-graph

- **Graph properties are per-link-label, not per-graph.** Since a Link can carry
  more than one meaning, "no cycles" is probably false globally and true per
  label: decomposition must be acyclic, "supersedes" forms chains, "cites" is
  many-to-one by design, "see also" may cycle harmlessly. Each label declares
  the algebra it must satisfy.
- **`/references` and distributability — an owner call, not a tool decision.**
  Whether a reference Concept holds identity and metadata (stable id, title,
  paragraph ref, URL, retrieval date) or mirrors the prose itself determines
  whether a Bundle of copyrighted standards can be a public repo. The
  hidden-edge benefit comes from the node existing and being addressable, not
  from it containing the text. Flagged because it decides repo visibility, which
  is structural. The tools do not decide it; people use them as they need.
- The graph carries node bodies as **opaque payloads** and does not interpret
  them.

## okf-normative

- **A ledger with a checker, not an inference engine.** Normative input comes
  from expert users, via an interactive mode that prompts for it. Agents assist;
  they do not decide.
- **Deriving stays on the table, and is distinct from inferring.** Computing an
  interior node's color from expert-declared leaves is arithmetic over
  declarations, not guesswork. It is what would let the tool *catch* a
  mis-coloring rather than merely notice that two declarations disagree — the
  strongest available answer to "declaration, not derivation".
- **open** — how the normative layer is managed; whether a hole can be a
  *frontier* that decomposes into another Concept (which changes a justification
  trace's stopping rule from "stop at every non-mechanical node" to "stop at
  every one no other Concept decomposes").

## Knowledge Bundles

- **An authority is not a regime.** Within IFRS, IAS 17 is superseded by IFRS 16.
  Regime is really *(corpus, standard, effective period)*. The third component is
  a time dimension nothing currently expresses, and it is where transition
  guidance lives (ASC 840 → 842, modified retrospective application).
- A capability this unlocks: **cross-corpus comparison of the same concept** —
  "under IFRS 16 the lessee classification norm does not exist; under ASC 840 it
  does." Dual reporters and GAAP/IFRS reconciliation are a real audience.
- **Fidelity is not machine-checkable.** No tool here verifies that a Bundle
  faithfully represents the standard it claims to encode. Expert users supply the
  normative content and are expected to review Bodies too: this is a
  code-agent-human interaction with a non-trivial onus on the human to engage
  properly. Worth stating loudly, because structural validation plus a machine
  author otherwise yields a Bundle that is well-formed, fully cited, internally
  consistent, and wrong — everything green, nothing true.

## Applied project

- **Owns evaluation**: mapping transactions to Concepts, resolving judgment holes
  with citations, computing the measurement layer, handing plain data to the
  kernel.
- Log files, immutable existing FrontMatter, and the validators as gates are what
  make agent-maintained Bundles safe. Those three read as housekeeping
  individually and as a design only together.

## What this architecture settles

- **The evaluation contradiction.** deon's DESIGN describes its value as "given a
  proposed set of seam inputs …" while §9 disclaims execution. That is a misfiled
  requirement, not a contradiction: evaluation is the application's layer. The
  tools stay representational permanently, and without apology.
- **The measurement layer.** A commitment is a *policy selection*
  (`classification: liability`); the kernel consumes *numbers plus a
  classification*. IAS 32's compound instruments make it concrete — one
  instrument becomes two classified accounts, split by an allocation. That
  arithmetic belongs to the application, rather than being missing between two
  repos.
- **Scope.** deon insisted it owned "the language, not the norms" while
  accumulating accounting examples. Bundles resolve it: the tools keep only
  fixtures.

## How we build

pacioli's approach: **small, deliberate steps aimed at a robust and
well-understood set of methods rather than at features.** Prefer the change that
makes the model clearer over the one that adds surface. Record why, and record
what was rejected.

Where OKF fights us, **log the concrete case when it happens** — an eventual
conversation with that community should rest on a dated list of real friction,
not recollection. Taking anything upstream is the owner's call.

The graph layer's assumptions about OKF should be **enumerable rather than
implicit** — a stated profile, each item pointing at the code that enforces it —
so friction points are legible rather than buried in `if` statements.

## Transition notes

- The repo rename is a **repo-settings change**, so it runs through the
  `~/github-settings` seat rather than from a working session.
- Check whether `.github/rulesets/main.json`, `scripts/settings.sh`, or
  `scripts/merge.sh` hardcode the repo name before it flips.
- **`CLAUDE.md` needs rewriting early, not late.** It is the first thing every
  session reads, and it currently describes a single-crate accounting-judgment
  repo. This session was bitten twice by stale claims in it.

## Open, deliberately

Colour derivation over the graph; holes as frontiers; bundle-wide (not per-file)
consistency; typing commitments at the seam so `classification: vibes` stops
passing; the time dimension; how the normative layer is managed; whether a second
evaluation ever appears (none can be named today, which is why the crates are not
built as a plugin architecture).
