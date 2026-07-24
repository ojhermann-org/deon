//! **Topology and identity** of an OKF Knowledge Bundle read as a knowledge
//! graph: what Concepts exist, what they are called, what points at what, what
//! resolves and what dangles.
//!
//! Node bodies are carried as **opaque payloads** and are not interpreted here.
//! This crate knows nothing about colors, norms, obligations, or the
//! mechanical/judgment seam — that is [`okf-normative`]'s work, and it depends
//! on this crate rather than the other way around.
//!
//! The boundary is easy to violate inside a workspace, so here is a falsifiable
//! test for it: **if this crate's test suite cannot be read by someone who has
//! never heard of deontic logic, the boundary is not real.**
//!
//! Graph properties are checked **per link label**, not per graph. A Link may
//! carry more than one meaning, and the algebra differs by meaning: decomposition
//! must be acyclic, "supersedes" forms chains, "cites" is many-to-one by design,
//! "see also" may cycle harmlessly. A global acyclicity rule would reject a
//! correct Bundle.
//!
//! So far this crate models the unit the graph is made of: an OKF
//! [`Concept`] — one markdown document, read as a [`Frontmatter`] block and a
//! [`Body`] (SPEC §4). Bundles, ids, links, and resolution come next.
//!
//! [`okf-normative`]: https://github.com/ojhermann-org/okf-tools

mod concept;

pub use concept::{Body, Concept, ConceptError, Frontmatter};
