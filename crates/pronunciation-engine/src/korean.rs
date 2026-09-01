use std::fmt;

const HANGUL_BASE: u32 = 0xAC00;
const HANGUL_LAST: u32 = 0xD7A3;
const NUCLEUS_COUNT: u32 = 21;
const CODA_COUNT: u32 = 28;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Consonant {
    G,
    GG,
    K,
    N,
    D,
    DD,
    T,
    R,
    M,
    B,
    BB,
    P,
    S,
    SS,
    J,
    JJ,
    C,
    NG,
    H,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Vowel {
    A,
    Ae,
    Ya,
    Yae,
    Eo,
    E,
    Yeo,
    Ye,
    O,
    Wa,
    Wae,
    Oe,
    Yo,
    U,
    Weo,
    We,
    Wi,
    Yu,
    Eu,
    Ui,
    I,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Coda {
    Single(Consonant),
    Double(Consonant, Consonant),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Syllable {
    pub onset: Option<Consonant>,
    pub nucleus: Vowel,
    pub coda: Option<Coda>,
    pub word_boundary_before: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrthographicWord {
    pub original: String,
    pub syllables: Vec<Syllable>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ParseError {
    EmptyInput,
    UnsupportedCharacter { character: char, byte_index: usize },
}

pub(crate) fn parse(input: &str) -> Result<OrthographicWord, ParseError> {
    let mut syllables = Vec::new();
    let mut boundary_pending = true;

    for (byte_index, character) in input.char_indices() {
        if character.is_whitespace() {
            boundary_pending = true;
            continue;
        }

        let code = character as u32;
        if !(HANGUL_BASE..=HANGUL_LAST).contains(&code) {
            return Err(ParseError::UnsupportedCharacter {
                character,
                byte_index,
            });
        }

        let offset = code - HANGUL_BASE;
        let onset_index = offset / (NUCLEUS_COUNT * CODA_COUNT);
        let nucleus_index = (offset % (NUCLEUS_COUNT * CODA_COUNT)) / CODA_COUNT;
        let coda_index = offset % CODA_COUNT;

        syllables.push(Syllable {
            onset: onset(onset_index),
            nucleus: nucleus(nucleus_index),
            coda: coda(coda_index),
            word_boundary_before: std::mem::take(&mut boundary_pending),
        });
    }

    if syllables.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    Ok(OrthographicWord {
        original: input.to_owned(),
        syllables,
    })
}

pub(crate) fn describe(word: &OrthographicWord) -> String {
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

fn onset(index: u32) -> Option<Consonant> {
    use Consonant::*;
    Some(match index {
        0 => G,
        1 => GG,
        2 => N,
        3 => D,
        4 => DD,
        5 => R,
        6 => M,
        7 => B,
        8 => BB,
        9 => S,
        10 => SS,
        11 => None?,
        12 => J,
        13 => JJ,
        14 => C,
        15 => K,
        16 => T,
        17 => P,
        18 => H,
        _ => unreachable!("Hangul onset index is bounded"),
    })
}

fn nucleus(index: u32) -> Vowel {
    use Vowel::*;
    match index {
        0 => A,
        1 => Ae,
        2 => Ya,
        3 => Yae,
        4 => Eo,
        5 => E,
        6 => Yeo,
        7 => Ye,
        8 => O,
        9 => Wa,
        10 => Wae,
        11 => Oe,
        12 => Yo,
        13 => U,
        14 => Weo,
        15 => We,
        16 => Wi,
        17 => Yu,
        18 => Eu,
        19 => Ui,
        20 => I,
        _ => unreachable!("Hangul nucleus index is bounded"),
    }
}

fn coda(index: u32) -> Option<Coda> {
    use Coda::{Double, Single};
    use Consonant::*;
    Some(match index {
        0 => None?,
        1 => Single(G),
        2 => Single(GG),
        3 => Double(G, S),
        4 => Single(N),
        5 => Double(N, J),
        6 => Double(N, H),
        7 => Single(D),
        8 => Single(R),
        9 => Double(R, G),
        10 => Double(R, M),
        11 => Double(R, B),
        12 => Double(R, S),
        13 => Double(R, T),
        14 => Double(R, P),
        15 => Double(R, H),
        16 => Single(M),
        17 => Single(B),
        18 => Double(B, S),
        19 => Single(S),
        20 => Single(SS),
        21 => Single(NG),
        22 => Single(J),
        23 => Single(C),
        24 => Single(K),
        25 => Single(T),
        26 => Single(P),
        27 => Single(H),
        _ => unreachable!("Hangul coda index is bounded"),
    })
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
