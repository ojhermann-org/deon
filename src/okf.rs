//! **deon's bundle contract** — what the checker requires of the concept bundle
//! it consumes, and the binding that reads it out of OKF-format markdown.
//!
//! This is deliberately framed as *deon's* contract rather than as a provisional
//! stub of someone else's spec. Norm content lives beside its cited prose in a
//! bundle (see the scope rule: deon owns the language, not the norms), and the
//! checker needs exactly two things from it. OKF is the intended carrier and the
//! only binding implemented, but the requirement is deon's and does not wait on
//! an upstream format to settle — otherwise the two bundle-backed checks would
//! be blocked on a spec in another project's repo indefinitely.
//!
//! What deon requires of a bundle:
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
//! **The OKF binding.** A bundle is a directory of markdown concept files; the
//! two requirements above are read from `{#id}` anchors and from a file's YAML
//! frontmatter. That mapping is this module's only job, and nothing else in the
//! checker depends on it — when the OKF spec settles, or another carrier is
//! wanted, the binding changes here and the contract above does not.
//!
//! One consequence, decided rather than deferred (issue #20): a concept file
//! with **no frontmatter at all** declares no state space and is *not* a defect.
//! A prose-only concept file is ordinary, and nothing in deon's contract says a
//! carrier must put frontmatter on every file. Frontmatter that is present but
//! unparseable, or a subject with no `states:` list, is a different matter — the
//! file is announcing a declaration and yielding none, which is COVER-4.

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
    defects: Vec<BundleDefect>,
}

/// A place the bundle failed to yield a state space that it looks like it meant
/// to. Retained rather than skipped: an unreadable declaration block drops
/// states silently, and a state absent from the space is one coverage stops
/// looking for (issue #18).
#[derive(Debug, Clone)]
pub(crate) struct BundleDefect {
    pub(crate) file: String,
    pub(crate) path: String,
    pub(crate) detail: String,
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

    /// Every place the bundle failed to yield a state space it looks like it
    /// meant to declare.
    pub(crate) fn defects(&self) -> &[BundleDefect] {
        &self.defects
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
        states_in(&text, &path.display().to_string(), out);
    }
    Ok(())
}

/// Read the `subjects: { <name>: { states: [...] } }` block from a concept
/// file's frontmatter, recording both what it declares and where it failed to.
/// Loading never fails: the bundle format is provisional, and a located finding
/// says more than a load error does.
///
/// A file with **no frontmatter** declares no state spaces and is not a defect —
/// a prose-only concept file is ordinary, and telling one apart from a file that
/// meant to declare a state space needs the OKF format to settle whether
/// frontmatter is mandatory (issue #20). Frontmatter that is present but
/// *unparseable* is unambiguous, and so is a `subjects:` entry whose `states:`
/// is missing or is not a list: both look like a declaration and yield nothing.
fn states_in(text: &str, file: &str, out: &mut Okf) {
    let Some(front) = frontmatter(text) else {
        return;
    };
    let doc = match serde_yaml::from_str::<Value>(front) {
        Ok(doc) => doc,
        Err(e) => {
            out.defects.push(BundleDefect {
                file: file.to_string(),
                path: "frontmatter".to_string(),
                detail: format!(
                    "concept file opens with a `---` fence but its frontmatter does not \
                     parse ({e}) — any state space it declares is silently invisible"
                ),
            });
            return;
        }
    };
    let Some(Value::Mapping(subjects)) = doc.get("subjects") else {
        return;
    };
    for (name, body) in subjects {
        let subject = key_str(name);
        let Some(Value::Sequence(states)) = body.get("states") else {
            out.defects.push(BundleDefect {
                file: file.to_string(),
                path: format!("subjects.{subject}.states"),
                detail: format!(
                    "subject `{subject}` is declared with no `states:` list — it yields an \
                     empty state space, so coverage has nothing to check against"
                ),
            });
            continue;
        };
        for (index, state) in states.iter().enumerate() {
            let grounds = state.get("grounds");
            let decl = StateDecl {
                file: file.to_string(),
                subject: subject.clone(),
                index,
                id: state_id(state),
                reference: grounds.and_then(|g| str_field(g, "ref")),
                source: grounds.and_then(|g| str_field(g, "source")),
            };
            if let Some(id) = &decl.id {
                out.states
                    .entry(subject.clone())
                    .or_default()
                    .insert(id.clone());
            }
            out.declarations.push(decl);
        }
    }
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
