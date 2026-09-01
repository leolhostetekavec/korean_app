---
artifact_repo: korean_app
---

# Language Content

The second project part contains language islands, lessons, phrases, and related learning content. Its sentence content is stored as JSON owned by the Rust application crate under crates/app; it is not a third crate.

LanguageContent remains a separate intent entity from Application so its learning-content meaning can evolve independently from app behavior. Its components, contracts, JSON schema, and exact data paths have not yet been specified and remain intentionally open.
