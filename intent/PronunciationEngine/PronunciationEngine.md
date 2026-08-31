---
artifact_repo: korean_app
---

# Pronunciation Engine

The pronunciation engine converts Korean Hangul into a French-native-friendly pronunciation representation. Its goal is practical Korean pronunciation for a native French reader, not romanization.

The engine is composed of four independent components:

- parser: Hangul blocks to Korean orthographic structure;
- rules: Korean orthography to Korean surface pronunciation;
- mapper: Korean surface sounds to French-oriented phonetic targets;
- renderer: French-oriented targets to readable output.

The mandatory direction is:

```text
Hangul
→ Korean orthographic structure
→ Korean phonological pronunciation
→ French-oriented phonetic approximation
→ French-readable pronunciation
```

French adaptation must never determine Korean phonology. The Korean pronunciation stage exposes a stable interface so other language-specific renderers can be added later.

V1 targets modern standard Korean at a practical learner-oriented level. Dialects, historical pronunciation, casual-speech contractions, emotional speech, speaker-specific phonetics, microscopic coarticulation, acoustic simulation, and exhaustive lexical exceptions are outside scope.

The implementation should be developed in this order: parser, Korean representations, resyllabification, coda processing, assimilation, ㅎ and aspiration, palatalization, tensification, Korean surface output, French mapping, and French rendering. Each stage must be independently testable and should expose intermediate results for diagnosis.

