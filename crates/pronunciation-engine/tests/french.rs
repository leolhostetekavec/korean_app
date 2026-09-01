use pronunciation_engine::{pronounce_for_french, trace_pronunciation};

#[test]
fn preserves_plain_fortis_and_aspirated_contrasts() {
    let plain = pronounce_for_french("가").unwrap();
    let fortis = pronounce_for_french("까").unwrap();
    let aspirated = pronounce_for_french("카").unwrap();
    assert_ne!(plain, fortis);
    assert_ne!(plain, aspirated);
    assert_ne!(fortis, aspirated);
}

#[test]
fn renders_french_oriented_vowels() {
    assert_eq!(pronounce_for_french("우으어").unwrap(), "ou-eu-euh");
}

#[test]
fn keeps_korean_and_french_stages_separate() {
    let trace = trace_pronunciation("강").unwrap();
    assert!(trace.korean_surface.contains("NG"));
    assert!(trace.french_target.contains("Ng"));
    assert_eq!(trace.rendered, "kang");
}

#[test]
fn adapts_plain_stops_by_word_position() {
    assert_eq!(pronounce_for_french("가").unwrap(), "ka");
    assert_eq!(pronounce_for_french("아가").unwrap(), "a-ga");
}

#[test]
fn resolves_ui_in_the_french_mapper() {
    assert_eq!(pronounce_for_french("의").unwrap(), "eui");
    assert_eq!(pronounce_for_french("희").unwrap(), "hi");
}

#[test]
fn rendering_is_deterministic() {
    let first = pronounce_for_french("한국어").unwrap();
    let second = pronounce_for_french("한국어").unwrap();
    assert_eq!(first, second);
}
