//! A minimal, **provisional** OKF concept bundle — the two things the checker
//! needs from norm content that lives beside its cited prose.
//!
//! The real OKF format (a directory of markdown concept files with YAML
//! frontmatter, one concept per file) is upstream and not yet settled, so this
//! reads only what the bundle-backed checks need:
//!
//! - the set of **anchor ids** a bundle declares, for GROUND-3. An anchor is
//!   declared by a trailing `{#id}` on any line (the pandoc/markdown-it
//!   convention the seed refs like `#ifrs15-35a` presume);
//! - each subject's **state space**, for coverage (check 3), read from a
//!   concept file's own frontmatter:
//!
//! ```yaml
//! subjects:
//!   performance-obligation:
//!     states:
//!       - { id: not-yet-satisfied, grounds: { ref: "#ifrs15-31", source: standard-criterion } }
//!       - { id: satisfied-over-time, grounds: { ref: "#ifrs15-35", source: standard-criterion } }
//! ```
//!
//! A state may also be written as a bare string. That form still *counts* as a
//! declaration — dropping it would quietly shrink the state space, which is the
//! failure COVER-3 exists to catch — but it can carry no citation, so it always
//! trips GROUND-1. Parse leniently, judge strictly. Which states a subject has is
//! itself a judgment about the standard, so it is norm content and belongs here
//! beside the prose — not in the checker, and not in a norm file. Being a
//! judgment, it must cite: each declaration is kept with *where* it was written
//! and *how* it grounds, so coverage can hold the bundle to the same standard a
//! norm file is held to (issue #18).
//!
//! Swap this module when the OKF spec lands; nothing else in the checker
//! depends on the format.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_yaml::Value;

use crate::{frontmatter, key_str, str_field};

/// What a bundle declares: anchor ids, and each subject's state space.
#[derive(Debug, Clone, Default)]
pub struct Okf {
    anchors: BTreeSet<String>,
    states: BTreeMap<String, BTreeSet<String>>,
    declarations: Vec<StateDecl>,
}

/// One state declaration, as written: which concept file and subject it belongs
/// to, the id it declares (absent if malformed), and its citation. Kept so the
/// declaration can be checked, not merely read.
#[derive(Debug, Clone)]
pub(crate) struct StateDecl {
    pub(crate) file: String,
    pub(crate) subject: String,
    pub(crate) index: usize,
    pub(crate) id: Option<String>,
    pub(crate) reference: Option<String>,
    pub(crate) source: Option<String>,
}

impl Okf {
    /// Load a bundle from a path: a directory is searched recursively for
    /// `*.md`; a single file is read as-is.
    pub fn load(path: &Path) -> std::io::Result<Okf> {
        let mut okf = Okf::default();
        collect(path, &mut okf)?;
        Ok(okf)
    }

    /// Does `reference` (with or without a leading `#`) resolve to a declared
    /// anchor?
    pub fn resolves(&self, reference: &str) -> bool {
        self.anchors.contains(reference.trim_start_matches('#'))
    }

    /// Number of anchors declared (for reporting).
    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    /// Whether the bundle declares no anchors.
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }

    /// The states declared for `subject`, if the bundle declares a state space
    /// for it at all. `None` and an empty set are different: the first means
    /// "this bundle says nothing about the subject's states", which is what
    /// coverage needs to know before it can flag a gap.
    pub(crate) fn states(&self, subject: &str) -> Option<&BTreeSet<String>> {
        self.states.get(subject)
    }

    /// Number of subjects with a declared state space (for reporting).
    pub fn subjects(&self) -> usize {
        self.states.len()
    }

    /// Every state declaration the bundle carries, in load order.
    pub(crate) fn declarations(&self) -> &[StateDecl] {
        &self.declarations
    }
}

fn collect(path: &Path, out: &mut Okf) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            collect(&entry?.path(), out)?;
        }
    } else if path.extension().is_some_and(|e| e == "md") {
        let text = std::fs::read_to_string(path)?;
        for id in anchors_in(&text) {
            out.anchors.insert(id);
        }
        for decl in states_in(&text, &path.display().to_string()) {
            let entry = out.states.entry(decl.subject.clone()).or_default();
            if let Some(id) = &decl.id {
                entry.insert(id.clone());
            }
            out.declarations.push(decl);
        }
    }
    Ok(())
}

/// Read the `subjects: { <name>: { states: [...] } }` block from a concept
/// file's frontmatter. A file with no frontmatter, no `subjects:`, or
/// unparseable YAML simply declares no state spaces — the bundle format is
/// provisional, so this never fails the load.
fn states_in(text: &str, file: &str) -> Vec<StateDecl> {
    let Some(front) = frontmatter(text) else {
        return Vec::new();
    };
    let Ok(doc) = serde_yaml::from_str::<Value>(front) else {
        return Vec::new();
    };
    let Some(Value::Mapping(subjects)) = doc.get("subjects") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (name, body) in subjects {
        let Some(Value::Sequence(states)) = body.get("states") else {
            continue;
        };
        for (index, state) in states.iter().enumerate() {
            let grounds = state.get("grounds");
            out.push(StateDecl {
                file: file.to_string(),
                subject: key_str(name),
                index,
                id: state_id(state),
                reference: grounds.and_then(|g| str_field(g, "ref")),
                source: grounds.and_then(|g| str_field(g, "source")),
            });
        }
    }
    out
}

/// A state is either `{ id: <name>, grounds: ... }` or a bare `<name>`.
fn state_id(state: &Value) -> Option<String> {
    match state {
        Value::String(s) => Some(s.clone()),
        _ => str_field(state, "id"),
    }
}

/// Extract every `{#id}` anchor declared in a concept file.
fn anchors_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find("{#") {
        rest = &rest[open + 2..];
        if let Some(close) = rest.find('}') {
            let id = rest[..close].trim();
            if !id.is_empty() {
                out.push(id.to_string());
            }
            rest = &rest[close + 1..];
        } else {
            break;
        }
    }
    out
}
