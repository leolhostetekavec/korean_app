use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::path::{Component, Path};

pub const SUPPORTED_BUNDLE_VERSION: u32 = 1;
pub const TTS_PROVIDER: &str = "Google Cloud Text-to-Speech";
pub const TTS_MODEL: &str = "Chirp 3: HD";

/// Reports whether an audio reference resolves in the supplied course assets.
pub trait AudioAssetCatalog {
    fn contains(&self, audio_ref: &str) -> bool;
}

/// Immutable course content in its supplied presentation order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CourseBundle {
    version: u32,
    objectives: Vec<String>,
    islands: Vec<Island>,
}

impl CourseBundle {
    pub fn import_json(
        json: &str,
        audio_assets: &impl AudioAssetCatalog,
    ) -> Result<Self, ImportError> {
        let raw: RawBundle = serde_json::from_str(json)
            .map_err(|error| ImportError::InvalidJson(error.to_string()))?;
        raw.validate(audio_assets)
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn objectives(&self) -> &[String] {
        &self.objectives
    }

    pub fn islands(&self) -> &[Island] {
        &self.islands
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Island {
    id: String,
    objectives: Vec<String>,
    lessons: Vec<Lesson>,
}

impl Island {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn objectives(&self) -> &[String] {
        &self.objectives
    }

    pub fn lessons(&self) -> &[Lesson] {
        &self.lessons
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lesson {
    id: String,
    objectives: Vec<String>,
    sentences: Vec<Sentence>,
}

impl Lesson {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn objectives(&self) -> &[String] {
        &self.objectives
    }

    pub fn sentences(&self) -> &[Sentence] {
        &self.sentences
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sentence {
    id: String,
    french_prompt: String,
    korean_text: String,
    french_pronunciation: String,
    audio_ref: String,
}

impl Sentence {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn french_prompt(&self) -> &str {
        &self.french_prompt
    }

    pub fn korean_text(&self) -> &str {
        &self.korean_text
    }

    pub fn french_pronunciation(&self) -> &str {
        &self.french_pronunciation
    }

    pub fn audio_ref(&self) -> &str {
        &self.audio_ref
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportError {
    InvalidJson(String),
    UnsupportedVersion(u32),
    InvalidTtsSource {
        provider: String,
        model: String,
    },
    DuplicateIdentity {
        kind: &'static str,
        id: String,
    },
    EmptyField {
        owner: String,
        field: &'static str,
    },
    DuplicateOrderIdentity {
        lesson_id: String,
        sentence_id: String,
    },
    UnknownOrderIdentity {
        lesson_id: String,
        sentence_id: String,
    },
    MissingOrderIdentity {
        lesson_id: String,
        sentence_id: String,
    },
    InvalidAudioReference {
        sentence_id: String,
        audio_ref: String,
    },
    MissingAudioAsset {
        sentence_id: String,
        audio_ref: String,
    },
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(formatter, "invalid course JSON: {message}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported course bundle version {version}")
            }
            Self::InvalidTtsSource { provider, model } => {
                write!(formatter, "unsupported TTS source {provider:?} / {model:?}")
            }
            Self::DuplicateIdentity { kind, id } => {
                write!(formatter, "duplicate {kind} identity {id:?}")
            }
            Self::EmptyField { owner, field } => {
                write!(formatter, "{owner} has an empty {field} field")
            }
            Self::DuplicateOrderIdentity {
                lesson_id,
                sentence_id,
            } => write!(
                formatter,
                "lesson {lesson_id:?} orders sentence {sentence_id:?} more than once"
            ),
            Self::UnknownOrderIdentity {
                lesson_id,
                sentence_id,
            } => write!(
                formatter,
                "lesson {lesson_id:?} orders unknown sentence {sentence_id:?}"
            ),
            Self::MissingOrderIdentity {
                lesson_id,
                sentence_id,
            } => write!(
                formatter,
                "lesson {lesson_id:?} does not order sentence {sentence_id:?}"
            ),
            Self::InvalidAudioReference {
                sentence_id,
                audio_ref,
            } => write!(
                formatter,
                "sentence {sentence_id:?} has invalid audio reference {audio_ref:?}"
            ),
            Self::MissingAudioAsset {
                sentence_id,
                audio_ref,
            } => write!(
                formatter,
                "sentence {sentence_id:?} references missing audio asset {audio_ref:?}"
            ),
        }
    }
}

impl Error for ImportError {}

#[derive(Deserialize)]
struct RawBundle {
    bundle_version: u32,
    tts: RawTts,
    #[serde(default)]
    objectives: Vec<String>,
    islands: Vec<RawIsland>,
}

impl RawBundle {
    fn validate(self, audio_assets: &impl AudioAssetCatalog) -> Result<CourseBundle, ImportError> {
        if self.bundle_version != SUPPORTED_BUNDLE_VERSION {
            return Err(ImportError::UnsupportedVersion(self.bundle_version));
        }
        if self.tts.provider != TTS_PROVIDER || self.tts.model != TTS_MODEL {
            return Err(ImportError::InvalidTtsSource {
                provider: self.tts.provider,
                model: self.tts.model,
            });
        }

        validate_strings("course bundle", "objective", &self.objectives)?;

        let mut island_ids = HashSet::new();
        let mut lesson_ids = HashSet::new();
        let mut sentence_ids = HashSet::new();
        let islands = self
            .islands
            .into_iter()
            .map(|island| {
                island.validate(
                    audio_assets,
                    &mut island_ids,
                    &mut lesson_ids,
                    &mut sentence_ids,
                )
            })
            .collect::<Result<_, _>>()?;

        Ok(CourseBundle {
            version: self.bundle_version,
            objectives: self.objectives,
            islands,
        })
    }
}

#[derive(Deserialize)]
struct RawTts {
    provider: String,
    model: String,
}

#[derive(Deserialize)]
struct RawIsland {
    island_id: String,
    #[serde(default)]
    objectives: Vec<String>,
    lessons: Vec<RawLesson>,
}

impl RawIsland {
    fn validate(
        self,
        audio_assets: &impl AudioAssetCatalog,
        island_ids: &mut HashSet<String>,
        lesson_ids: &mut HashSet<String>,
        sentence_ids: &mut HashSet<String>,
    ) -> Result<Island, ImportError> {
        validate_required("island", "island_id", &self.island_id)?;
        insert_identity("island", &self.island_id, island_ids)?;
        validate_strings(
            &format!("island {:?}", self.island_id),
            "objective",
            &self.objectives,
        )?;

        let lessons = self
            .lessons
            .into_iter()
            .map(|lesson| lesson.validate(audio_assets, lesson_ids, sentence_ids))
            .collect::<Result<_, _>>()?;

        Ok(Island {
            id: self.island_id,
            objectives: self.objectives,
            lessons,
        })
    }
}

#[derive(Deserialize)]
struct RawLesson {
    lesson_id: String,
    #[serde(default)]
    objectives: Vec<String>,
    sentence_order: Vec<String>,
    sentences: UniqueMap<RawSentence>,
}

impl RawLesson {
    fn validate(
        self,
        audio_assets: &impl AudioAssetCatalog,
        lesson_ids: &mut HashSet<String>,
        sentence_ids: &mut HashSet<String>,
    ) -> Result<Lesson, ImportError> {
        validate_required("lesson", "lesson_id", &self.lesson_id)?;
        insert_identity("lesson", &self.lesson_id, lesson_ids)?;
        validate_strings(
            &format!("lesson {:?}", self.lesson_id),
            "objective",
            &self.objectives,
        )?;

        let mut sentences: HashMap<_, _> = self.sentences.into_iter().collect();
        let mut ordered_ids = HashSet::new();
        let mut ordered_sentences = Vec::with_capacity(sentences.len());

        for sentence_id in self.sentence_order {
            if !ordered_ids.insert(sentence_id.clone()) {
                return Err(ImportError::DuplicateOrderIdentity {
                    lesson_id: self.lesson_id,
                    sentence_id,
                });
            }
            let raw_sentence = sentences.remove(&sentence_id).ok_or_else(|| {
                ImportError::UnknownOrderIdentity {
                    lesson_id: self.lesson_id.clone(),
                    sentence_id: sentence_id.clone(),
                }
            })?;
            if !sentence_ids.insert(sentence_id.clone()) {
                return Err(ImportError::DuplicateIdentity {
                    kind: "sentence",
                    id: sentence_id,
                });
            }
            ordered_sentences.push(raw_sentence.validate(sentence_id, audio_assets)?);
        }

        if let Some(sentence_id) = sentences.into_keys().min() {
            return Err(ImportError::MissingOrderIdentity {
                lesson_id: self.lesson_id,
                sentence_id,
            });
        }

        Ok(Lesson {
            id: self.lesson_id,
            objectives: self.objectives,
            sentences: ordered_sentences,
        })
    }
}

#[derive(Deserialize)]
struct RawSentence {
    french_prompt: String,
    korean_text: String,
    french_pronunciation: String,
    audio_ref: String,
}

impl RawSentence {
    fn validate(
        self,
        sentence_id: String,
        audio_assets: &impl AudioAssetCatalog,
    ) -> Result<Sentence, ImportError> {
        let owner = format!("sentence {sentence_id:?}");
        validate_required(&owner, "french_prompt", &self.french_prompt)?;
        validate_required(&owner, "korean_text", &self.korean_text)?;
        validate_required(&owner, "french_pronunciation", &self.french_pronunciation)?;
        validate_required(&owner, "audio_ref", &self.audio_ref)?;

        if !is_safe_audio_ref(&self.audio_ref) {
            return Err(ImportError::InvalidAudioReference {
                sentence_id,
                audio_ref: self.audio_ref,
            });
        }
        if !audio_assets.contains(&self.audio_ref) {
            return Err(ImportError::MissingAudioAsset {
                sentence_id,
                audio_ref: self.audio_ref,
            });
        }

        Ok(Sentence {
            id: sentence_id,
            french_prompt: self.french_prompt,
            korean_text: self.korean_text,
            french_pronunciation: self.french_pronunciation,
            audio_ref: self.audio_ref,
        })
    }
}

fn validate_required(owner: &str, field: &'static str, value: &str) -> Result<(), ImportError> {
    if value.trim().is_empty() {
        return Err(ImportError::EmptyField {
            owner: owner.to_owned(),
            field,
        });
    }
    Ok(())
}

fn validate_strings(
    owner: &str,
    field: &'static str,
    values: &[String],
) -> Result<(), ImportError> {
    for value in values {
        validate_required(owner, field, value)?;
    }
    Ok(())
}

fn insert_identity(
    kind: &'static str,
    id: &str,
    identities: &mut HashSet<String>,
) -> Result<(), ImportError> {
    if !identities.insert(id.to_owned()) {
        return Err(ImportError::DuplicateIdentity {
            kind,
            id: id.to_owned(),
        });
    }
    Ok(())
}

fn is_safe_audio_ref(audio_ref: &str) -> bool {
    if audio_ref.contains('\\') {
        return false;
    }
    let mut components = Path::new(audio_ref).components();
    matches!(components.next(), Some(Component::Normal(first)) if first == "audio")
        && matches!(components.next(), Some(Component::Normal(_)))
        && components.all(|component| matches!(component, Component::Normal(_)))
}

struct UniqueMap<T>(Vec<(String, T)>);

impl<T> IntoIterator for UniqueMap<T> {
    type Item = (String, T);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'de, T> Deserialize<'de> for UniqueMap<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UniqueMapVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for UniqueMapVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = UniqueMap<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map with unique sentence identities")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                let mut identities = HashSet::new();
                while let Some((id, sentence)) = map.next_entry::<String, T>()? {
                    if !identities.insert(id.clone()) {
                        return Err(de::Error::custom(format!(
                            "duplicate sentence identity {id:?}"
                        )));
                    }
                    entries.push((id, sentence));
                }
                Ok(UniqueMap(entries))
            }
        }

        deserializer.deserialize_map(UniqueMapVisitor(PhantomData))
    }
}
