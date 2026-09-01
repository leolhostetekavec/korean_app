use pronunciation_engine::{PronunciationError, trace_pronunciation};

#[test]
fn decomposes_zero_onset_and_final_ng_differently() {
    let trace = trace_pronunciation("어강").unwrap();
    assert!(trace.parsed.contains("None,Eo,None"));
    assert!(trace.parsed.contains("Some(G),A,Some(Single(NG))"));
}

#[test]
fn preserves_every_complex_coda_as_a_pair() {
    let expected = [
        (3, "Double(G, S)"),
        (5, "Double(N, J)"),
        (6, "Double(N, H)"),
        (9, "Double(R, G)"),
        (10, "Double(R, M)"),
        (11, "Double(R, B)"),
        (12, "Double(R, S)"),
        (13, "Double(R, T)"),
        (14, "Double(R, P)"),
        (15, "Double(R, H)"),
        (18, "Double(B, S)"),
    ];

    for (coda_index, representation) in expected {
        let syllable = char::from_u32(0xAC00 + 11 * 21 * 28 + coda_index).unwrap();
        let trace = trace_pronunciation(&syllable.to_string()).unwrap();
        assert!(trace.parsed.contains(representation));
    }
}

#[test]
fn accepts_every_modern_onset_vowel_and_coda_index() {
    for onset_index in 0..19 {
        let syllable = char::from_u32(0xAC00 + onset_index * 21 * 28).unwrap();
        trace_pronunciation(&syllable.to_string()).unwrap();
    }
    for vowel_index in 0..21 {
        let syllable = char::from_u32(0xAC00 + 11 * 21 * 28 + vowel_index * 28).unwrap();
        trace_pronunciation(&syllable.to_string()).unwrap();
    }
    for coda_index in 0..28 {
        let syllable = char::from_u32(0xAC00 + 11 * 21 * 28 + coda_index).unwrap();
        trace_pronunciation(&syllable.to_string()).unwrap();
    }
}

#[test]
fn preserves_word_boundaries_without_creating_syllables() {
    let trace = trace_pronunciation("한 국").unwrap();
    assert_eq!(trace.parsed.matches('|').count(), 2);
}

#[test]
fn rejects_non_hangul_with_a_structured_location() {
    assert_eq!(
        trace_pronunciation("한a").unwrap_err(),
        PronunciationError::UnsupportedCharacter {
            character: 'a',
            byte_index: 3,
        }
    );
}

#[test]
fn rejects_empty_or_whitespace_only_input() {
    assert_eq!(
        trace_pronunciation("  \n").unwrap_err(),
        PronunciationError::EmptyInput
    );
}
