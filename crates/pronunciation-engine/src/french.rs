use crate::korean::Vowel;
use crate::rules::{CodaSound, ConsonantSound, KoreanWord};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OnsetTarget {
    KInitial,
    GPlain,
    GFortis,
    GAspirated,
    N,
    TInitial,
    DPlain,
    DFortis,
    DAspirated,
    Flap,
    Lateral,
    M,
    PInitial,
    BPlain,
    BFortis,
    BAspirated,
    SPlain,
    SFortis,
    ChInitial,
    JPlain,
    JFortis,
    JAspirated,
    Ng,
    H,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VowelTarget {
    A,
    EAcute,
    Ya,
    YeAcute,
    Euh,
    Yeuh,
    O,
    Wa,
    Wae,
    We,
    Yo,
    Ou,
    Wo,
    Wi,
    You,
    Eu,
    Eui,
    I,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CodaTarget {
    KUnreleased,
    TUnreleased,
    PUnreleased,
    N,
    M,
    Ng,
    L,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TargetSyllable {
    onset: Option<OnsetTarget>,
    vowel: VowelTarget,
    coda: Option<CodaTarget>,
    word_boundary_before: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrenchTarget {
    syllables: Vec<TargetSyllable>,
}

pub(crate) fn map(word: &KoreanWord) -> FrenchTarget {
    FrenchTarget {
        syllables: word
            .syllables
            .iter()
            .map(|syllable| TargetSyllable {
                onset: syllable.onset.map(map_onset),
                vowel: map_vowel(
                    syllable.nucleus,
                    syllable.word_boundary_before,
                    syllable.onset.is_none(),
                ),
                coda: syllable.coda.map(map_coda),
                word_boundary_before: syllable.word_boundary_before,
            })
            .collect(),
    }
}

pub(crate) fn render(target: &FrenchTarget) -> String {
    let mut output = String::new();
    for (index, syllable) in target.syllables.iter().enumerate() {
        if index > 0 {
            output.push(if syllable.word_boundary_before {
                ' '
            } else {
                '-'
            });
        }
        if let Some(onset) = syllable.onset {
            output.push_str(render_onset(onset));
        }
        output.push_str(render_vowel(syllable.vowel));
        if let Some(coda) = syllable.coda {
            output.push_str(render_coda(coda));
        }
    }
    output
}

pub(crate) fn describe(target: &FrenchTarget) -> String {
    format!("{:?}", target.syllables)
}

fn map_onset(sound: ConsonantSound) -> OnsetTarget {
    match sound {
        ConsonantSound::GInitial => OnsetTarget::KInitial,
        ConsonantSound::G => OnsetTarget::GPlain,
        ConsonantSound::GFortis => OnsetTarget::GFortis,
        ConsonantSound::GAspirated => OnsetTarget::GAspirated,
        ConsonantSound::N => OnsetTarget::N,
        ConsonantSound::DInitial => OnsetTarget::TInitial,
        ConsonantSound::D => OnsetTarget::DPlain,
        ConsonantSound::DFortis => OnsetTarget::DFortis,
        ConsonantSound::DAspirated => OnsetTarget::DAspirated,
        ConsonantSound::RFlap => OnsetTarget::Flap,
        ConsonantSound::RLateral => OnsetTarget::Lateral,
        ConsonantSound::M => OnsetTarget::M,
        ConsonantSound::BInitial => OnsetTarget::PInitial,
        ConsonantSound::B => OnsetTarget::BPlain,
        ConsonantSound::BFortis => OnsetTarget::BFortis,
        ConsonantSound::BAspirated => OnsetTarget::BAspirated,
        ConsonantSound::S => OnsetTarget::SPlain,
        ConsonantSound::SFortis => OnsetTarget::SFortis,
        ConsonantSound::JInitial => OnsetTarget::ChInitial,
        ConsonantSound::J => OnsetTarget::JPlain,
        ConsonantSound::JFortis => OnsetTarget::JFortis,
        ConsonantSound::JAspirated => OnsetTarget::JAspirated,
        ConsonantSound::NG => OnsetTarget::Ng,
        ConsonantSound::H => OnsetTarget::H,
    }
}

fn map_vowel(vowel: Vowel, word_initial: bool, zero_onset: bool) -> VowelTarget {
    use Vowel::*;
    match vowel {
        A => VowelTarget::A,
        Ae | E => VowelTarget::EAcute,
        Ya => VowelTarget::Ya,
        Yae | Ye => VowelTarget::YeAcute,
        Eo => VowelTarget::Euh,
        Yeo => VowelTarget::Yeuh,
        O => VowelTarget::O,
        Wa => VowelTarget::Wa,
        Wae => VowelTarget::Wae,
        Oe | We => VowelTarget::We,
        Yo => VowelTarget::Yo,
        U => VowelTarget::Ou,
        Weo => VowelTarget::Wo,
        Wi => VowelTarget::Wi,
        Yu => VowelTarget::You,
        Eu => VowelTarget::Eu,
        Ui if word_initial && zero_onset => VowelTarget::Eui,
        Ui => VowelTarget::I,
        I => VowelTarget::I,
    }
}

fn map_coda(coda: CodaSound) -> CodaTarget {
    match coda {
        CodaSound::K => CodaTarget::KUnreleased,
        CodaSound::T => CodaTarget::TUnreleased,
        CodaSound::P => CodaTarget::PUnreleased,
        CodaSound::N => CodaTarget::N,
        CodaSound::M => CodaTarget::M,
        CodaSound::NG => CodaTarget::Ng,
        CodaSound::R => CodaTarget::L,
    }
}

fn render_onset(onset: OnsetTarget) -> &'static str {
    use OnsetTarget::*;
    match onset {
        KInitial => "k",
        GPlain => "g",
        GFortis => "k’",
        GAspirated => "k-h",
        N => "n",
        TInitial => "t",
        DPlain => "d",
        DFortis => "t’",
        DAspirated => "t-h",
        Flap => "r̆",
        Lateral => "l",
        M => "m",
        PInitial => "p",
        BPlain => "b",
        BFortis => "p’",
        BAspirated => "p-h",
        SPlain => "s",
        SFortis => "ss",
        ChInitial => "tch",
        JPlain => "dj",
        JFortis => "tch’",
        JAspirated => "tch-h",
        Ng => "ng",
        H => "h",
    }
}

fn render_vowel(vowel: VowelTarget) -> &'static str {
    use VowelTarget::*;
    match vowel {
        A => "a",
        EAcute => "é",
        Ya => "ya",
        YeAcute => "yé",
        Euh => "euh",
        Yeuh => "yeuh",
        O => "o",
        Wa => "wa",
        Wae => "waé",
        We => "wé",
        Yo => "yo",
        Ou => "ou",
        Wo => "wo",
        Wi => "wi",
        You => "you",
        Eu => "eu",
        Eui => "eui",
        I => "i",
    }
}

fn render_coda(coda: CodaTarget) -> &'static str {
    use CodaTarget::*;
    match coda {
        KUnreleased => "k̚",
        TUnreleased => "t̚",
        PUnreleased => "p̚",
        N => "n",
        M => "m",
        Ng => "ng",
        L => "l",
    }
}
