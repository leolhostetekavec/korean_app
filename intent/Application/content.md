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

## CourseBundleContract

- binding: semantic
- artifact_path: crates/app/src/content.rs

The V1 course import must:

1. Accept a versioned course bundle containing islands, lessons, sentence content, and referenced audio assets.
2. Represent each lesson's sentences as a keyed map whose key is the stable sentence identity.
3. Preserve sentence presentation order through an explicit sentence-order list rather than relying on map iteration order.
4. Require each sentence record to provide a French prompt, authoritative Korean text, French-friendly pronunciation, and an audio reference.
5. Treat the authoritative Korean text as stored content that remains hidden on V1 learning, practice, and revision cards.
6. Resolve each audio reference to validated Google Cloud Chirp 3: HD-generated course audio that is bundled with or available to the course bundle.
7. Preserve island order, lesson order, sentence order, grouping, and stable identities when importing the bundle.
8. Reject a bundle with an unsupported version, duplicate identity, missing required field, invalid order reference, missing audio asset, or invalid audio reference.
9. Keep imported course content immutable during learning and store learner progress separately, keyed by the stable sentence identity.
10. Never generate TTS, rewrite sentence content, or reinterpret pronunciation during runtime import.

The transport shape for a lesson is:

```json
{
  "sentence_order": ["sentence-001"],
  "sentences": {
    "sentence-001": {
      "french_prompt": "Hello",
      "korean_text": "안녕하세요",
      "french_pronunciation": "ann-yong-ha-se-yo",
      "audio_ref": "audio/sentence-001.mp3"
    }
  }
}
```
