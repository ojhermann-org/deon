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
//! A state may also be written as a bare string. Which states a subject has is
//! itself a judgment about the standard, so it is norm content and belongs here
//! beside the prose — not in the checker, and not in a norm file. Their
//! `grounds` are carried but not yet validated (GROUND runs over norm files);
//! extending it to the bundle is follow-up work.
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
        for (subject, states) in states_in(&text) {
            out.states.entry(subject).or_default().extend(states);
        }
    }
    Ok(())
}

/// Read the `subjects: { <name>: { states: [...] } }` block from a concept
/// file's frontmatter. A file with no frontmatter, no `subjects:`, or
/// unparseable YAML simply declares no state spaces — the bundle format is
/// provisional, so this never fails the load.
fn states_in(text: &str) -> Vec<(String, BTreeSet<String>)> {
    let Some(front) = frontmatter(text) else {
        return Vec::new();
    };
    let Ok(doc) = serde_yaml::from_str::<Value>(front) else {
        return Vec::new();
    };
    let Some(Value::Mapping(subjects)) = doc.get("subjects") else {
        return Vec::new();
    };
    subjects
        .iter()
        .map(|(name, body)| {
            let states = match body.get("states") {
                Some(Value::Sequence(s)) => s.iter().filter_map(state_id).collect(),
                _ => BTreeSet::new(),
            };
            (key_str(name), states)
        })
        .collect()
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
