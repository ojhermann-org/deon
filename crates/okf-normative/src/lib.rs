//! **Semantics** over an [`okf_graph`]: the normative reading of a Knowledge
//! Bundle — the mechanical / judgment / election cut, grounding, coverage,
//! conflict, defeat, and termination at the seam.
//!
//! Consumes a validated graph and never re-parses a Bundle. Interprets the
//! payloads that [`okf_graph`] carries but does not read.
//!
//! Two commitments carried over from the archived `deon-check` crate, which is
//! this crate's reference implementation:
//!
//! - **A ledger with a checker, not an inference engine.** Normative input comes
//!   from expert users; agents assist, they do not decide. Whether something is
//!   mechanical is a standard-setter's policy choice, not a discoverable
//!   property, so the tool records and checks declarations rather than guessing.
//! - **Deriving is not inferring.** Computing an interior node's color from
//!   expert-declared leaves is arithmetic over declarations, and stays on the
//!   table — it is what would let the checker *catch* a mis-coloring rather than
//!   merely notice that two declarations disagree.
//!
//! Nothing is implemented yet.
