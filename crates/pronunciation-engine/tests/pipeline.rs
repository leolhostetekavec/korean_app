use pronunciation_engine::{PronunciationError, pronounce_for_french, trace_pronunciation};

#[test]
fn runs_the_complete_pipeline() {
    assert_eq!(pronounce_for_french("한국어").unwrap(), "han-gou-geuh");
}

#[test]
fn handles_multisyllabic_learner_input() {
    let rendered = pronounce_for_french("먹어요").unwrap();
    assert_eq!(rendered, "meuh-geuh-yo");
}

#[test]
fn does_not_apply_rules_across_spaces() {
    let joined = pronounce_for_french("국어").unwrap();
    let separated = pronounce_for_french("국 어").unwrap();
    assert_ne!(joined, separated);
    assert!(separated.contains(' '));
}

#[test]
fn trace_contains_every_public_diagnostic_projection() {
    let trace = trace_pronunciation("한").unwrap();
    assert_eq!(trace.input, "한");
    assert!(!trace.parsed.is_empty());
    assert_eq!(trace.korean_stages.len(), 10);
    assert!(!trace.korean_surface.is_empty());
    assert!(!trace.french_target.is_empty());
    assert_eq!(trace.rendered, "han");
}

#[test]
fn reports_invalid_input_instead_of_guessing() {
    assert!(matches!(
        pronounce_for_french("한국!"),
        Err(PronunciationError::UnsupportedCharacter { character: '!', .. })
    ));
}
