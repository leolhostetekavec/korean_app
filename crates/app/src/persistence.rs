use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const STATE_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplicationState {
    state_version: u32,
    pub completed_lessons: BTreeSet<String>,
    pub completed_islands: BTreeSet<String>,
    pub current_lesson: Option<String>,
    pub sentence_states: BTreeMap<String, SentenceLearningState>,
    pub practice_pools: BTreeMap<String, Vec<String>>,
    pub revision_records: BTreeMap<String, RevisionRecord>,
    pub unfinished_lesson: Option<UnfinishedLesson>,
    pub unfinished_revision_session: Option<UnfinishedRevisionSession>,
    pub settings: PersistedSettings,
}

impl ApplicationState {
    pub fn version(&self) -> u32 {
        self.state_version
    }

    fn validate(&self) -> Result<(), StateValidationError> {
        if self.settings.default_revision_session_size == 0 {
            return Err(StateValidationError::ZeroRevisionSessionSize);
        }
        if self.settings.mastery_threshold == 0 {
            return Err(StateValidationError::ZeroMasteryThreshold);
        }
        if self.settings.audio_volume_percent > 100 {
            return Err(StateValidationError::InvalidAudioVolume(
                self.settings.audio_volume_percent,
            ));
        }

        for (sentence_id, record) in &self.revision_records {
            if sentence_id != &record.sentence_id {
                return Err(StateValidationError::RevisionIdentityMismatch {
                    key: sentence_id.clone(),
                    record: record.sentence_id.clone(),
                });
            }
        }

        if let Some(lesson) = &self.unfinished_lesson
            && lesson.practice_position > lesson.practice_order.len()
        {
            return Err(StateValidationError::InvalidPracticePosition);
        }
        if let Some(session) = &self.unfinished_revision_session
            && (session.sentence_order.is_empty()
                || session.current_position >= session.sentence_order.len())
        {
            return Err(StateValidationError::InvalidRevisionPosition);
        }

        Ok(())
    }
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            state_version: STATE_VERSION,
            completed_lessons: BTreeSet::new(),
            completed_islands: BTreeSet::new(),
            current_lesson: None,
            sentence_states: BTreeMap::new(),
            practice_pools: BTreeMap::new(),
            revision_records: BTreeMap::new(),
            unfinished_lesson: None,
            unfinished_revision_session: None,
            settings: PersistedSettings::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SentenceLearningState {
    New,
    Practice,
    Active,
    Mastered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecallOutcome {
    Known,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RevisionStatus {
    Active,
    Mastered,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RevisionRecord {
    pub sentence_id: String,
    pub last_revised: u64,
    pub revision_count: u32,
    pub status: RevisionStatus,
    pub successful_revision_history: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnfinishedLesson {
    pub lesson_id: String,
    pub next_introduction_position: usize,
    pub outcomes: BTreeMap<String, RecallOutcome>,
    pub practice_order: Vec<String>,
    pub practice_position: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnfinishedRevisionSession {
    pub sentence_order: Vec<String>,
    pub current_position: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersistedSettings {
    pub default_revision_session_size: usize,
    pub mastery_threshold: u32,
    pub audio_volume_percent: u8,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            default_revision_session_size: 10,
            mastery_threshold: 10,
            audio_volume_percent: 100,
        }
    }
}

pub trait StateStore {
    type Error;

    fn read(&self) -> Result<Option<Vec<u8>>, Self::Error>;
    fn write(&self, bytes: &[u8]) -> Result<(), Self::Error>;
}

pub struct Persistence<S> {
    store: S,
}

impl<S> Persistence<S>
where
    S: StateStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn load(&self) -> Result<ApplicationState, PersistenceError<S::Error>> {
        let Some(bytes) = self.store.read().map_err(PersistenceError::Store)? else {
            return Ok(ApplicationState::default());
        };

        let header: StateHeader = serde_json::from_slice(&bytes)
            .map_err(|error| PersistenceError::InvalidState(error.to_string()))?;
        if header.state_version != STATE_VERSION {
            return Err(PersistenceError::UnsupportedVersion(header.state_version));
        }

        let state: ApplicationState = serde_json::from_slice(&bytes)
            .map_err(|error| PersistenceError::InvalidState(error.to_string()))?;
        state.validate().map_err(PersistenceError::InvalidData)?;
        Ok(state)
    }

    pub fn save(&self, state: &ApplicationState) -> Result<(), PersistenceError<S::Error>> {
        state.validate().map_err(PersistenceError::InvalidData)?;
        let bytes = serde_json::to_vec(state)
            .map_err(|error| PersistenceError::InvalidState(error.to_string()))?;
        self.store.write(&bytes).map_err(PersistenceError::Store)
    }
}

#[derive(Deserialize)]
struct StateHeader {
    state_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateValidationError {
    ZeroRevisionSessionSize,
    ZeroMasteryThreshold,
    InvalidAudioVolume(u8),
    RevisionIdentityMismatch { key: String, record: String },
    InvalidPracticePosition,
    InvalidRevisionPosition,
}

impl fmt::Display for StateValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRevisionSessionSize => {
                formatter.write_str("default revision session size must be greater than zero")
            }
            Self::ZeroMasteryThreshold => {
                formatter.write_str("mastery threshold must be greater than zero")
            }
            Self::InvalidAudioVolume(volume) => {
                write!(formatter, "audio volume {volume} is outside 0..=100")
            }
            Self::RevisionIdentityMismatch { key, record } => write!(
                formatter,
                "revision record key {key:?} does not match sentence identity {record:?}"
            ),
            Self::InvalidPracticePosition => {
                formatter.write_str("unfinished practice position is outside its order")
            }
            Self::InvalidRevisionPosition => {
                formatter.write_str("unfinished revision position is outside its order")
            }
        }
    }
}

impl Error for StateValidationError {}

#[derive(Debug)]
pub enum PersistenceError<E> {
    Store(E),
    InvalidState(String),
    UnsupportedVersion(u32),
    InvalidData(StateValidationError),
}

impl<E> fmt::Display for PersistenceError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "local state storage failed: {error}"),
            Self::InvalidState(error) => write!(formatter, "saved state is invalid: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "saved state version {version} is not supported")
            }
            Self::InvalidData(error) => write!(formatter, "saved state data is invalid: {error}"),
        }
    }
}

impl<E> Error for PersistenceError<E> where E: Error + 'static {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileStateStore {
    path: PathBuf,
}

impl FileStateStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn temporary_path(&self) -> PathBuf {
        let mut name = self
            .path
            .file_name()
            .map(OsString::from)
            .unwrap_or_else(|| OsString::from("application-state"));
        name.push(".tmp");
        self.path.with_file_name(name)
    }
}

impl StateStore for FileStateStore {
    type Error = io::Error;

    fn read(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn write(&self, bytes: &[u8]) -> Result<(), Self::Error> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let temporary_path = self.temporary_path();
        let write_result = (|| {
            let mut file = File::create(&temporary_path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::rename(&temporary_path, &self.path)
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result
    }
}
