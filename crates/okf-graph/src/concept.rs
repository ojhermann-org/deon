//! An OKF **concept document** ([SPEC §4][spec]): one markdown file, read as a
//! YAML [`Frontmatter`] block and a markdown [`Body`].
//!
//! The split is the spec's, and it is the whole model — a [`Concept`] has
//! exactly those two parts. What it deliberately does *not* carry is its
//! **concept id**: that is "the path of the concept's file within the bundle,
//! with the `.md` suffix removed" (§2), which is a fact about the bundle, not
//! about the document. A file read on its own has no id, and inventing one from
//! the filesystem path would make two clones of the same bundle disagree about
//! identity. Identity belongs to whatever loads a bundle.
//!
//! **Parse leniently, judge strictly.** OKF's consumption model is permissive
//! (§9): a consumer must not reject a bundle over a missing optional field, an
//! unknown `type`, an unknown extra key, or a broken link. So parsing here fails
//! on one thing only — a file whose two parts cannot be told apart, or whose
//! metadata block cannot be read as metadata at all. Whatever the block *says*
//! is never a parse failure. Everything else is representable, and therefore
//! reportable: a concept
//! with no `type` (a §9 conformance failure) parses fine and answers `None`,
//! because a checker that cannot construct a defective document cannot say
//! anything located about it. A caller that walks a bundle knows the file name
//! and can turn a [`ConceptError`] into a located finding; it cannot recover a
//! document the parser refused to build.
//!
//! **Extension keys survive.** §4.1 lets producers add any keys they like and
//! asks consumers to preserve unknown ones. The typed accessors below cover the
//! six fields §4.1 names; everything else is kept verbatim in
//! [`Frontmatter::source`], which is the payload a semantic layer reads. This
//! crate does not interpret it.
//!
//! [spec]: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#4-concept-documents

use std::fmt;

use serde_yaml::{Mapping, Value};

/// One OKF concept document: its frontmatter and its body ([SPEC §4][spec]).
///
/// [spec]: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#4-concept-documents
#[derive(Debug, Clone, PartialEq)]
pub struct Concept {
    frontmatter: Frontmatter,
    body: Body,
}

impl Concept {
    /// Read a concept document from the text of one markdown file.
    ///
    /// The document must open with a `---` fence on its first line and close it
    /// on a later one; the text between is parsed as YAML, and *everything*
    /// after the closing fence is the body, verbatim. The ways that can fail are
    /// named by [`ConceptError`], and none of them is about content.
    pub fn parse(source: &str) -> Result<Concept, ConceptError> {
        let (front, body) = split(source)?;
        Ok(Concept {
            frontmatter: Frontmatter::parse(front)?,
            body: Body(body.to_string()),
        })
    }

    /// The document's frontmatter.
    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    /// The document's body.
    pub fn body(&self) -> &Body {
        &self.body
    }
}

/// The YAML metadata block at the top of a concept document
/// ([SPEC §4.1][spec]).
///
/// The accessors below cover the one required field and the five recommended
/// ones. They share a single rule: each answers `Some` only when the key is
/// present **and** holds the shape §4.1 describes, and `None` otherwise. That
/// conflates "absent" with "present but the wrong shape" on purpose — both mean
/// "this concept states no title", and neither is this type's to judge. The
/// parsed frontmatter is kept whole, so a conformance check can tell the two
/// apart later; an accessor that guessed would be the thing to regret.
///
/// [spec]: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#41-frontmatter
#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
    source: String,
    fields: Mapping,
}

impl Frontmatter {
    /// Parse a frontmatter block from the YAML text between the `---` fences.
    fn parse(source: &str) -> Result<Frontmatter, ConceptError> {
        let value = serde_yaml::from_str::<Value>(source)
            .map_err(|e| ConceptError::MalformedFrontmatter(e.to_string()))?;
        let fields = match value {
            Value::Mapping(fields) => fields,
            // An empty block — `---` immediately followed by `---` — parses as
            // null. It is a frontmatter block that declares nothing, which is
            // exactly an empty mapping; it fails §9 conformance for want of a
            // `type`, and that is a finding, not a parse error.
            Value::Null => Mapping::new(),
            _ => return Err(ConceptError::FrontmatterNotAMapping),
        };
        Ok(Frontmatter {
            source: source.to_string(),
            fields,
        })
    }

    /// `type` — the one **required** field: a short string naming the kind of
    /// concept, used for routing, filtering, and presentation.
    ///
    /// Named `concept_type` because `type` is a Rust keyword. Values are not
    /// registered anywhere and an unknown one is not an error (§4.1); `None`
    /// means the document fails §9 conformance, which is a checker's finding to
    /// report rather than a reason to refuse the document.
    pub fn concept_type(&self) -> Option<&str> {
        self.string("type")
    }

    /// `title` — human-readable display name. A consumer may derive one from
    /// the filename when it is absent.
    pub fn title(&self) -> Option<&str> {
        self.string("title")
    }

    /// `description` — a single sentence summarizing the concept.
    pub fn description(&self) -> Option<&str> {
        self.string("description")
    }

    /// `resource` — a URI identifying the underlying asset this concept
    /// describes. Absent for concepts that describe abstract ideas rather than
    /// physical resources.
    pub fn resource(&self) -> Option<&str> {
        self.string("resource")
    }

    /// `timestamp` — ISO 8601 datetime of the last meaningful change, kept
    /// exactly as written. Whether it *is* a valid ISO 8601 datetime is a
    /// conformance question about the string, not about the document's shape.
    pub fn timestamp(&self) -> Option<&str> {
        self.string("timestamp")
    }

    /// `tags` — the list of short strings used for cross-cutting
    /// categorization, or `None` when the key is absent or is not a list of
    /// strings.
    ///
    /// A list with a non-string entry answers `None` rather than the strings it
    /// happens to contain: silently dropping an entry would shrink a tag set
    /// without saying so, and a tag that quietly disappears is one nothing will
    /// ever look for again.
    pub fn tags(&self) -> Option<Vec<&str>> {
        match self.fields.get("tags") {
            Some(Value::Sequence(items)) => items
                .iter()
                .map(|item| match item {
                    Value::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect(),
            _ => None,
        }
    }

    /// The frontmatter exactly as written, fences excluded.
    ///
    /// Producers may add any keys they like and consumers are asked to preserve
    /// the ones they do not recognize (§4.1), so the block is kept verbatim
    /// rather than reduced to the fields above. This is the payload a semantic
    /// layer reads; this crate carries it and does not interpret it.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The string at `key`, if the frontmatter holds a string there.
    fn string(&self, key: &str) -> Option<&str> {
        match self.fields.get(key) {
            Some(Value::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// Everything in a concept document after the frontmatter ([SPEC §4.2][spec]):
/// standard markdown, carried verbatim.
///
/// It is an opaque payload here. The body is where a concept's links to other
/// concepts live (§5), so it is where this crate's topology will eventually be
/// read from — but reading it is a separate job from modelling the document,
/// and nothing here parses it.
///
/// [spec]: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md#42-body
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body(String);

impl Body {
    /// The body text as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The three ways a markdown file can fail to be a concept document.
///
/// All three are about **shape**, not content: they are the cases where the two
/// parts §4 defines cannot be told apart. Nothing about a concept's fields
/// appears here — a missing `type`, an unknown `type`, an unknown extra key,
/// and a broken link are all things §9 requires a consumer to tolerate, and a
/// checker cannot report on a document it refused to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConceptError {
    /// The file does not open with a `---` fence, so it has no frontmatter.
    MissingFrontmatter,
    /// The opening `---` fence is never closed, so where the frontmatter ends
    /// and the body begins is unknowable.
    UnterminatedFrontmatter,
    /// The frontmatter is not parseable YAML. Carries the parser's message.
    MalformedFrontmatter(String),
    /// The frontmatter parses, but as a scalar or a list — frontmatter is a
    /// block of key/value metadata, and a document whose block is a bare string
    /// declares no fields at all.
    FrontmatterNotAMapping,
}

impl fmt::Display for ConceptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConceptError::MissingFrontmatter => {
                write!(
                    f,
                    "no frontmatter: the file does not open with a `---` fence"
                )
            }
            ConceptError::UnterminatedFrontmatter => {
                write!(
                    f,
                    "unterminated frontmatter: the opening `---` fence is never closed"
                )
            }
            ConceptError::MalformedFrontmatter(e) => {
                write!(f, "frontmatter is not parseable YAML: {e}")
            }
            ConceptError::FrontmatterNotAMapping => {
                write!(f, "frontmatter is not a mapping, so it declares no fields")
            }
        }
    }
}

impl std::error::Error for ConceptError {}

/// Split a source file into its frontmatter text and its body.
///
/// The opening fence must be the first line — a `---` further down is a
/// horizontal rule in someone's prose, not the start of metadata. The closing
/// fence is the first later line that is `---` on its own, and the body starts
/// on the line after it. Both fences tolerate trailing whitespace, so a
/// CRLF-terminated file splits like any other.
fn split(source: &str) -> Result<(&str, &str), ConceptError> {
    let mut lines = source.split_inclusive('\n');
    let opening = lines.next().ok_or(ConceptError::MissingFrontmatter)?;
    if opening.trim_end() != "---" {
        return Err(ConceptError::MissingFrontmatter);
    }
    let start = opening.len();
    let mut offset = start;
    for line in source[start..].split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Ok((&source[start..offset], &source[offset + line.len()..]));
        }
        offset += line.len();
    }
    Err(ConceptError::UnterminatedFrontmatter)
}
