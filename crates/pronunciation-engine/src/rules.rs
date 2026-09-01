use crate::korean::{Coda, Consonant, OrthographicWord, Vowel};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsonantSound {
    GInitial,
    G,
    GFortis,
    GAspirated,
    N,
    DInitial,
    D,
    DFortis,
    DAspirated,
    RFlap,
    RLateral,
    M,
    BInitial,
    B,
    BFortis,
    BAspirated,
    S,
    SFortis,
    JInitial,
    J,
    JFortis,
    JAspirated,
    NG,
    H,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CodaSound {
    K,
    T,
    P,
    N,
    M,
    NG,
    R,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KoreanSyllable {
    pub onset: Option<ConsonantSound>,
    pub nucleus: Vowel,
    pub coda: Option<CodaSound>,
    pub word_boundary_before: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KoreanWord {
    pub syllables: Vec<KoreanSyllable>,
}

pub(crate) struct Analysis {
    pub word: KoreanWord,
    pub stages: Vec<Stage>,
}

pub(crate) struct Stage {
    pub name: &'static str,
    pub representation: String,
}

#[derive(Clone, Debug)]
struct WorkingSyllable {
    onset: Option<Consonant>,
    nucleus: Vowel,
    underlying_coda: Option<Coda>,
    coda: Option<Coda>,
    coda_sound: Option<CodaSound>,
    word_boundary_before: bool,
    resyllabified_onset: bool,
}

pub(crate) fn apply(word: &OrthographicWord) -> Analysis {
    let mut syllables = word
        .syllables
        .iter()
        .map(|syllable| WorkingSyllable {
            onset: syllable.onset,
            nucleus: syllable.nucleus,
            underlying_coda: syllable.coda,
            coda: syllable.coda,
            coda_sound: None,
            word_boundary_before: syllable.word_boundary_before,
            resyllabified_onset: false,
        })
        .collect::<Vec<_>>();
    let mut stages = Vec::with_capacity(10);

    capture(&mut stages, "boundaries", &syllables);
    resyllabify(&mut syllables);
    capture(&mut stages, "resyllabification", &syllables);
    resolve_complex_codas(&mut syllables);
    capture(&mut stages, "complex-coda-resolution", &syllables);
    neutralize_codas(&mut syllables);
    capture(&mut stages, "coda-neutralization", &syllables);
    assimilate_nasals(&mut syllables);
    capture(&mut stages, "nasal-assimilation", &syllables);
    assimilate_liquids(&mut syllables);
    capture(&mut stages, "liquid-rules", &syllables);
    apply_h_aspiration(&mut syllables);
    capture(&mut stages, "h-aspiration", &syllables);
    palatalize(&mut syllables);
    capture(&mut stages, "palatalization", &syllables);
    tensify(&mut syllables);
    capture(&mut stages, "tensification", &syllables);

    let surface = realize(syllables);
    stages.push(Stage {
        name: "contextual-onset-realization",
        representation: describe(&surface),
    });

    Analysis {
        word: surface,
        stages,
    }
}

pub(crate) fn describe(word: &KoreanWord) -> String {
    word.syllables
        .iter()
        .map(|syllable| {
            format!(
                "{}({:?},{:?},{:?})",
                if syllable.word_boundary_before {
                    "|"
                } else {
                    ""
                },
                syllable.onset,
                syllable.nucleus,
                syllable.coda
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capture(stages: &mut Vec<Stage>, name: &'static str, syllables: &[WorkingSyllable]) {
    stages.push(Stage {
        name,
        representation: format!("{syllables:?}"),
    });
}

fn resyllabify(syllables: &mut [WorkingSyllable]) {
    for index in 0..syllables.len().saturating_sub(1) {
        let (left, right) = split_pair(syllables, index);
        if right.word_boundary_before || right.onset.is_some() {
            continue;
        }

        match left.coda {
            Some(Coda::Single(Consonant::NG)) | None => {}
            Some(Coda::Single(Consonant::H)) => left.coda = None,
            Some(Coda::Single(consonant)) => {
                left.coda = None;
                right.onset = Some(consonant);
                right.resyllabified_onset = true;
            }
            Some(Coda::Double(first, Consonant::H)) => left.coda = Some(Coda::Single(first)),
            Some(Coda::Double(first, second)) => {
                left.coda = Some(Coda::Single(first));
                right.onset = Some(second);
                right.resyllabified_onset = true;
            }
        }
    }
}

fn resolve_complex_codas(syllables: &mut [WorkingSyllable]) {
    use Consonant::*;
    for syllable in syllables {
        let Some(Coda::Double(first, second)) = syllable.coda else {
            continue;
        };
        let survivor = match (first, second) {
            (G, S) => G,
            (N, J | H) => N,
            (R, G) => G,
            (R, M) => M,
            (R, B | S | T | H) => R,
            (R, P) => P,
            (B, S) => B,
            _ => first,
        };
        syllable.coda = Some(Coda::Single(survivor));
    }
}

fn neutralize_codas(syllables: &mut [WorkingSyllable]) {
    for syllable in syllables {
        syllable.coda_sound = syllable.coda.and_then(|coda| match coda {
            Coda::Single(consonant) => Some(neutral_coda(consonant)),
            Coda::Double(_, _) => None,
        });
    }
}

fn neutral_coda(consonant: Consonant) -> CodaSound {
    match consonant {
        Consonant::G | Consonant::GG | Consonant::K => CodaSound::K,
        Consonant::D
        | Consonant::T
        | Consonant::S
        | Consonant::SS
        | Consonant::J
        | Consonant::C
        | Consonant::H => CodaSound::T,
        Consonant::B | Consonant::P => CodaSound::P,
        Consonant::N => CodaSound::N,
        Consonant::R => CodaSound::R,
        Consonant::M => CodaSound::M,
        Consonant::NG => CodaSound::NG,
        Consonant::DD | Consonant::BB | Consonant::JJ => {
            unreachable!("fortis consonant cannot be an orthographic coda")
        }
    }
}

fn assimilate_nasals(syllables: &mut [WorkingSyllable]) {
    for index in 0..syllables.len().saturating_sub(1) {
        let (left, right) = split_pair(syllables, index);
        if right.word_boundary_before || !matches!(right.onset, Some(Consonant::N | Consonant::M)) {
            continue;
        }
        left.coda_sound = match left.coda_sound {
            Some(CodaSound::K) => Some(CodaSound::NG),
            Some(CodaSound::T) => Some(CodaSound::N),
            Some(CodaSound::P) => Some(CodaSound::M),
            other => other,
        };
    }
}

fn assimilate_liquids(syllables: &mut [WorkingSyllable]) {
    for index in 0..syllables.len().saturating_sub(1) {
        let (left, right) = split_pair(syllables, index);
        if right.word_boundary_before {
            continue;
        }
        match (left.coda_sound, right.onset) {
            (Some(CodaSound::N), Some(Consonant::R)) => {
                left.coda_sound = Some(CodaSound::R);
                right.onset = Some(Consonant::R);
            }
            (Some(CodaSound::R), Some(Consonant::N)) => right.onset = Some(Consonant::R),
            _ => {}
        }
    }
}

fn apply_h_aspiration(syllables: &mut [WorkingSyllable]) {
    for index in 0..syllables.len().saturating_sub(1) {
        let (left, right) = split_pair(syllables, index);
        if right.word_boundary_before {
            continue;
        }

        if right.onset == Some(Consonant::H) {
            let trigger = last_coda_consonant(left.underlying_coda);
            if let Some(aspirated) = trigger.and_then(aspirate) {
                right.onset = Some(aspirated);
                remove_last_coda_consonant(left);
            }
            continue;
        }

        if coda_has_h(left.underlying_coda)
            && let Some(aspirated) = right.onset.and_then(aspirate)
        {
            right.onset = Some(aspirated);
            remove_h_coda(left);
        }
    }
}

fn palatalize(syllables: &mut [WorkingSyllable]) {
    for syllable in syllables {
        if !syllable.resyllabified_onset || syllable.nucleus != Vowel::I {
            continue;
        }
        syllable.onset = match syllable.onset {
            Some(Consonant::D) => Some(Consonant::J),
            Some(Consonant::T) => Some(Consonant::C),
            other => other,
        };
    }
}

fn tensify(syllables: &mut [WorkingSyllable]) {
    for index in 0..syllables.len().saturating_sub(1) {
        let (left, right) = split_pair(syllables, index);
        if right.word_boundary_before
            || !matches!(
                left.coda_sound,
                Some(CodaSound::K | CodaSound::T | CodaSound::P)
            )
        {
            continue;
        }
        right.onset = match right.onset {
            Some(Consonant::G) => Some(Consonant::GG),
            Some(Consonant::D) => Some(Consonant::DD),
            Some(Consonant::B) => Some(Consonant::BB),
            Some(Consonant::S) => Some(Consonant::SS),
            Some(Consonant::J) => Some(Consonant::JJ),
            other => other,
        };
    }
}

fn realize(syllables: Vec<WorkingSyllable>) -> KoreanWord {
    let mut output = Vec::with_capacity(syllables.len());
    for (index, syllable) in syllables.iter().enumerate() {
        let previous_is_lateral = index > 0
            && !syllable.word_boundary_before
            && syllables[index - 1].coda_sound == Some(CodaSound::R);
        output.push(KoreanSyllable {
            onset: syllable.onset.map(|onset| {
                realize_onset(onset, syllable.word_boundary_before, previous_is_lateral)
            }),
            nucleus: syllable.nucleus,
            coda: syllable.coda_sound,
            word_boundary_before: syllable.word_boundary_before,
        });
    }
    KoreanWord { syllables: output }
}

fn realize_onset(consonant: Consonant, word_initial: bool, lateral: bool) -> ConsonantSound {
    match consonant {
        Consonant::G if word_initial => ConsonantSound::GInitial,
        Consonant::G => ConsonantSound::G,
        Consonant::GG => ConsonantSound::GFortis,
        Consonant::K => ConsonantSound::GAspirated,
        Consonant::N => ConsonantSound::N,
        Consonant::D if word_initial => ConsonantSound::DInitial,
        Consonant::D => ConsonantSound::D,
        Consonant::DD => ConsonantSound::DFortis,
        Consonant::T => ConsonantSound::DAspirated,
        Consonant::R if lateral => ConsonantSound::RLateral,
        Consonant::R => ConsonantSound::RFlap,
        Consonant::M => ConsonantSound::M,
        Consonant::B if word_initial => ConsonantSound::BInitial,
        Consonant::B => ConsonantSound::B,
        Consonant::BB => ConsonantSound::BFortis,
        Consonant::P => ConsonantSound::BAspirated,
        Consonant::S => ConsonantSound::S,
        Consonant::SS => ConsonantSound::SFortis,
        Consonant::J if word_initial => ConsonantSound::JInitial,
        Consonant::J => ConsonantSound::J,
        Consonant::JJ => ConsonantSound::JFortis,
        Consonant::C => ConsonantSound::JAspirated,
        Consonant::NG => ConsonantSound::NG,
        Consonant::H => ConsonantSound::H,
    }
}

fn aspirate(consonant: Consonant) -> Option<Consonant> {
    match consonant {
        Consonant::G => Some(Consonant::K),
        Consonant::D | Consonant::S | Consonant::SS => Some(Consonant::T),
        Consonant::B => Some(Consonant::P),
        Consonant::J => Some(Consonant::C),
        _ => None,
    }
}

fn last_coda_consonant(coda: Option<Coda>) -> Option<Consonant> {
    match coda {
        Some(Coda::Single(consonant)) => Some(consonant),
        Some(Coda::Double(_, second)) => Some(second),
        None => None,
    }
}

fn coda_has_h(coda: Option<Coda>) -> bool {
    matches!(
        coda,
        Some(Coda::Single(Consonant::H) | Coda::Double(_, Consonant::H))
    )
}

fn remove_h_coda(syllable: &mut WorkingSyllable) {
    match syllable.underlying_coda {
        Some(Coda::Single(Consonant::H)) => {
            syllable.coda = None;
            syllable.coda_sound = None;
        }
        Some(Coda::Double(first, Consonant::H)) => {
            syllable.coda = Some(Coda::Single(first));
            syllable.coda_sound = Some(neutral_coda(first));
        }
        _ => {}
    }
}

fn remove_last_coda_consonant(syllable: &mut WorkingSyllable) {
    match syllable.underlying_coda {
        Some(Coda::Single(_)) => {
            syllable.coda = None;
            syllable.coda_sound = None;
        }
        Some(Coda::Double(first, _)) => {
            syllable.coda = Some(Coda::Single(first));
            syllable.coda_sound = Some(neutral_coda(first));
        }
        None => {}
    }
}

fn split_pair(
    syllables: &mut [WorkingSyllable],
    index: usize,
) -> (&mut WorkingSyllable, &mut WorkingSyllable) {
    let (left, right) = syllables.split_at_mut(index + 1);
    (&mut left[index], &mut right[0])
}
