# Parser

The parser converts Unicode Hangul syllable blocks into an orthographic Korean representation. It records Korean identity and structure only; it does not apply pronunciation rules or French mappings.

## OrthographicRepresentationContract

- binding: semantic
- artifact_path: src/korean.rs

The parser must:

1. Accept Hangul syllable blocks as input and preserve their order in an orthographic word.
2. Decompose every syllable into one onset, one nucleus, and an optional coda.
3. Represent initial ㅇ as no onset, while retaining final ㅇ as the consonant identity NG.
4. Represent consonants by Korean identity: G, GG, K, N, D, DD, T, R, M, B, BB, P, S, SS, J, JJ, C, NG, and H.
5. Represent vowels by Korean identity: A, AE, YA, YAE, EO, E, YEO, YE, O, WA, WAE, OE, YO, U, WEO, WE, WI, YU, EU, UI, and I.
6. Preserve the distinction between an orthographic coda and a later phonological coda sound.
7. Preserve every consonant in a complex coda as ordered components rather than collapsing the cluster during parsing.
8. Represent complex codas as pairs for ㄳ, ㄵ, ㄶ, ㄺ, ㄻ, ㄼ, ㄽ, ㄾ, ㄿ, ㅀ, and ㅄ.
9. Reject or report unsupported non-Hangul input according to the repository's chosen error policy, without silently fabricating Korean structure.
10. Support inspection of the original Hangul alongside the parsed representation for debugging and stage-by-stage tests.

The conceptual data model is:

```rust
Syllable {
    onset: Option<Consonant>,
    nucleus: Vowel,
    coda: Option<Coda>,
}

OrthographicWord {
    syllables: Vec<Syllable>,
}
```

