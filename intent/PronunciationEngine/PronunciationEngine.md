---
artifact_repo: korean_app
---

# Pronunciation Engine

The pronunciation engine converts Korean Hangul into a French-native-friendly pronunciation representation. Its goal is practical Korean pronunciation for a native French reader, not romanization.

Its artifact is an independently importable Rust library crate under crates/pronunciation-engine. It is a private workspace dependency and must not be published to a public package registry. Consumers use its public pipeline API through a local path dependency.

The engine is composed of four independent components:

- parser: Hangul blocks to Korean orthographic structure;
- rules: Korean orthography to Korean surface pronunciation;
- mapper: Korean surface sounds to French-oriented phonetic targets;
- renderer: French-oriented targets to readable output.
- pipeline: public coordination of the complete conversion.

The mandatory direction is:

```text
Hangul
→ Korean orthographic structure
→ Korean phonological pronunciation
→ French-oriented phonetic approximation
→ French-readable pronunciation
```

French adaptation must never determine Korean phonology. The Korean pronunciation stage exposes a stable interface so other language-specific renderers can be added later.

The application crate consumes the complete pipeline through the library's public API. Internal parser, rule, mapping, and rendering details remain encapsulated unless a separate public use is intentionally introduced.

V1 targets modern standard Korean at a practical learner-oriented level. Dialects, historical pronunciation, casual-speech contractions, emotional speech, speaker-specific phonetics, microscopic coarticulation, acoustic simulation, and exhaustive lexical exceptions are outside scope.

The implementation should be developed in this order: parser, Korean representations, resyllabification, coda processing, assimilation, ㅎ and aspiration, palatalization, tensification, Korean surface output, French mapping, and French rendering. Each stage must be independently testable and should expose intermediate results for diagnosis.
