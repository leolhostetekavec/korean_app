---
tests_path: crates/pronunciation-engine/tests
---

# Korean Phonological Rules

This component transforms parsed Korean orthography into a Korean surface pronunciation. It must remain independent of French-oriented mapping and rendering.

## KoreanSurfaceContract

- binding: semantic
- artifact_path: crates/pronunciation-engine/src/rules.rs

The rule engine must:

1. Receive an orthographic word without destroying its source structure.
2. Preserve underlying coda identity until contextual rules that need it, especially liaison and complex-coda resolution, have run.
3. Apply transformations in an explicit, inspectable order: boundary detection, resyllabification, complex-coda resolution, coda neutralization, nasal assimilation, liquid rules, ㅎ and aspiration rules, palatalization, tensification, and contextual onset realization.
4. Resyllabify a coda before an orthographic zero onset when Korean phonology permits liaison, while distinguishing orthographic boundaries from pronounced boundaries.
5. Resolve complex codas contextually, including cases where only one component transfers to a following onset.
6. Neutralize remaining final consonants into K, T, P, N, M, NG, or R after contextual transfer and complex-coda resolution.
7. Represent final stops as unreleased Korean coda sounds rather than adding a French release or vowel.
8. Apply nasal assimilation so K, T, and P before a nasal environment become respectively NG, N, and M where Korean phonology requires it.
9. Keep underlying ㄹ as R, then realize it contextually as flap-like, lateral, or a lateral sequence; handle ㄴ plus ㄹ and ㄹ plus ㄴ lateral assimilation.
10. Apply ordered ㅎ interactions and aspiration, including the relevant plain-consonant plus ㅎ environments that yield aspirated Korean consonants.
11. Apply Korean palatalization for relevant ㄷ and ㅌ sequences before high front-vowel environments such as 이.
12. Apply tensification where Korean phonology requires a following plain consonant to become fortis.
13. Realize plain, aspirated, and fortis consonants as distinct Korean surface categories until the French mapper consumes them.
14. Preserve Korean vowel identities through this component, including the special identity EU for ㅡ and context-sensitive UI for ㅢ.
15. Expose intermediate results after each rule stage so rule ordering and regressions can be tested.

The conceptual Korean surface inventory includes plain, fortis, and aspirated variants for applicable stops and affricates, plus N, R, M, NG, S, H, and the coda categories K, T, P, N, M, NG, and R.
