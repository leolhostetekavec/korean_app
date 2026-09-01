---
tests_path: crates/app/tests
---

# Content Boundary

This component adapts content owned by LanguageContent for use by the application without taking ownership of its curriculum design.

## ContentBoundaryContract

- binding: semantic
- artifact_path: crates/app/src/content.rs

The content boundary must:

1. Accept the ordered islands, lessons, sentences, and learning objectives supplied by LanguageContent.
2. Preserve the supplied identities, ordering, grouping, and associations while presenting them to application flows.
3. Expose the French prompt, French-friendly pronunciation, Korean audio reference, and stable sentence identity required by the recall interaction.
4. Keep authoritative Hangul available to the content boundary without exposing it on V1 learning or revision cards.
5. Impose no application-owned number of islands, lessons, or sentences.
6. Impose no application-owned curriculum topology, sentence schema, topic composition, vocabulary policy, or difficulty progression.
7. Never generate, rewrite, or reinterpret the Korean sentence, its pronunciation, or its audio reference.
8. Report unavailable or unusable supplied content without fabricating replacement learning material.
