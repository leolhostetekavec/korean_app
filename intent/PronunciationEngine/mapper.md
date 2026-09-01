---
tests_path: crates/pronunciation-engine/tests
---

# French Phonetic Mapper

The mapper consumes only Korean surface pronunciation. It converts Korean sound categories into French-oriented phonetic targets without changing the Korean analysis.

## FrenchTargetContract

- binding: semantic
- artifact_path: crates/pronunciation-engine/src/french.rs

The mapper must:

1. Accept the stable Korean surface representation produced by the rules component.
2. Never inspect Hangul spelling to guess or override Korean pronunciation.
3. Preserve meaningful Korean plain, aspirated, and fortis distinctions as separate French-oriented targets until rendering.
4. Map Korean vowels only after Korean phonology is complete.
5. Treat ㅡ as a distinct Korean vowel input; use a declared practical French approximation such as an eu-like target without claiming exact equivalence.
6. Keep context-dependent vowels such as ㅢ resolvable by the mapper rather than hardwiring them into the parser or Korean rules.
7. Provide practical targets for core consonants including m, n, ng, p, t, k, h, flap-like ㄹ, and lateral ㄹ.
8. Keep the mapping convention explicit and replaceable so improved French approximations do not require changes to Korean parsing or phonology.
9. Produce an ordered French-oriented sound sequence suitable for the renderer and independently testable before spelling output is generated.
