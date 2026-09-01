use pronunciation_engine::trace_pronunciation;

#[test]
fn exposes_the_rule_order() {
    let trace = trace_pronunciation("한국어").unwrap();
    let names = trace
        .korean_stages
        .iter()
        .map(|stage| stage.stage)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "boundaries",
            "resyllabification",
            "complex-coda-resolution",
            "coda-neutralization",
            "nasal-assimilation",
            "liquid-rules",
            "h-aspiration",
            "palatalization",
            "tensification",
            "contextual-onset-realization",
        ]
    );
}

#[test]
fn resyllabifies_before_a_zero_onset() {
    let trace = trace_pronunciation("먹어").unwrap();
    let stage = &trace.korean_stages[1].representation;
    assert!(stage.contains("coda: None"));
    assert!(stage.contains("onset: Some(G)"));
}

#[test]
fn nasalizes_an_obstruent_before_m() {
    let trace = trace_pronunciation("국물").unwrap();
    let stage = trace
        .korean_stages
        .iter()
        .find(|stage| stage.stage == "nasal-assimilation")
        .unwrap();
    assert!(stage.representation.contains("coda_sound: Some(NG)"));
}

#[test]
fn palatalizes_a_resyllabified_t_before_i() {
    let trace = trace_pronunciation("같이").unwrap();
    let stage = trace
        .korean_stages
        .iter()
        .find(|stage| stage.stage == "palatalization")
        .unwrap();
    assert!(stage.representation.contains("onset: Some(C)"));
}

#[test]
fn aspirates_after_a_h_coda() {
    let trace = trace_pronunciation("좋다").unwrap();
    let stage = trace
        .korean_stages
        .iter()
        .find(|stage| stage.stage == "h-aspiration")
        .unwrap();
    assert!(stage.representation.contains("onset: Some(T)"));
}

#[test]
fn preserves_the_first_member_when_hidden_h_aspirates() {
    let trace = trace_pronunciation("많다").unwrap();
    let stage = trace
        .korean_stages
        .iter()
        .find(|stage| stage.stage == "h-aspiration")
        .unwrap();
    assert!(stage.representation.contains("coda_sound: Some(N)"));
    assert!(stage.representation.contains("onset: Some(T)"));
}

#[test]
fn tensifies_after_an_obstruent_coda() {
    let trace = trace_pronunciation("학교").unwrap();
    let stage = trace
        .korean_stages
        .iter()
        .find(|stage| stage.stage == "tensification")
        .unwrap();
    assert!(stage.representation.contains("onset: Some(GG)"));
}
