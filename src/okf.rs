//! A minimal, **provisional** OKF concept bundle — just enough to resolve
//! `grounds.ref` anchors for GROUND-3.
//!
//! The real OKF format (a directory of markdown concept files with YAML
//! frontmatter, one concept per file) is upstream and not yet settled, so this
//! reads only what anchor *resolution* needs: the set of anchor ids a bundle
//! declares. An anchor is declared by a trailing `{#id}` on any line (the
//! pandoc/markdown-it convention the seed refs like `#ifrs15-35a` presume).
//! Swap this module when the OKF spec lands; nothing else in the checker
//! depends on the format.

use std::collections::BTreeSet;
use std::path::Path;

/// The anchor ids a bundle declares.
#[derive(Debug, Clone, Default)]
pub struct Okf {
    anchors: BTreeSet<String>,
}

impl Okf {
    /// Load a bundle from a path: a directory is searched recursively for
    /// `*.md`; a single file is read as-is.
    pub fn load(path: &Path) -> std::io::Result<Okf> {
        let mut anchors = BTreeSet::new();
        collect(path, &mut anchors)?;
        Ok(Okf { anchors })
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
}

fn collect(path: &Path, out: &mut BTreeSet<String>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            collect(&entry?.path(), out)?;
        }
    } else if path.extension().is_some_and(|e| e == "md") {
        for id in anchors_in(&std::fs::read_to_string(path)?) {
            out.insert(id);
        }
    }
    Ok(())
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
