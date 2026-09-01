mod french;
mod korean;
mod rules;

use std::error::Error;
use std::fmt;

/// A complete, inspectable pronunciation run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PronunciationTrace {
    pub input: String,
    pub parsed: String,
    pub korean_stages: Vec<StageTrace>,
    pub korean_surface: String,
    pub french_target: String,
    pub rendered: String,
}

/// One named snapshot in the ordered Korean rule pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageTrace {
    pub stage: &'static str,
    pub representation: String,
}

/// Errors produced before a trustworthy pronunciation can be built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PronunciationError {
    EmptyInput,
    UnsupportedCharacter { character: char, byte_index: usize },
}

impl fmt::Display for PronunciationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("input contains no Hangul syllables"),
            Self::UnsupportedCharacter {
                character,
                byte_index,
            } => write!(
                formatter,
                "unsupported character {character:?} at byte index {byte_index}"
            ),
        }
    }
}

impl Error for PronunciationError {}

impl From<korean::ParseError> for PronunciationError {
    fn from(error: korean::ParseError) -> Self {
        match error {
            korean::ParseError::EmptyInput => Self::EmptyInput,
            korean::ParseError::UnsupportedCharacter {
                character,
                byte_index,
            } => Self::UnsupportedCharacter {
                character,
                byte_index,
            },
        }
    }
}

/// Converts Hangul into a deterministic French-native-readable pronunciation.
pub fn pronounce_for_french(input: &str) -> Result<String, PronunciationError> {
    Ok(trace_pronunciation(input)?.rendered)
}

/// Runs the same public pipeline while retaining every diagnostic stage.
pub fn trace_pronunciation(input: &str) -> Result<PronunciationTrace, PronunciationError> {
    let parsed = korean::parse(input)?;
    let analysis = rules::apply(&parsed);
    let target = french::map(&analysis.word);
    let rendered = french::render(&target);

    Ok(PronunciationTrace {
        input: parsed.original.clone(),
        parsed: korean::describe(&parsed),
        korean_stages: analysis
            .stages
            .into_iter()
            .map(|stage| StageTrace {
                stage: stage.name,
                representation: stage.representation,
            })
            .collect(),
        korean_surface: rules::describe(&analysis.word),
        french_target: french::describe(&target),
        rendered,
    })
}
