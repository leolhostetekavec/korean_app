use korean_app_core::persistence::{
    ApplicationState, FileStateStore, PersistedSettings, Persistence, PersistenceError,
    RecallOutcome, RevisionRecord, RevisionStatus, SentenceLearningState, StateStore,
    StateValidationError, UnfinishedLesson, UnfinishedRevisionSession,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Default)]
struct MemoryStore {
    bytes: Rc<RefCell<Option<Vec<u8>>>>,
}

impl MemoryStore {
    fn with_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: Rc::new(RefCell::new(Some(bytes.to_vec()))),
        }
    }
}

impl StateStore for MemoryStore {
    type Error = Infallible;

    fn read(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.bytes.borrow().clone())
    }

    fn write(&self, bytes: &[u8]) -> Result<(), Self::Error> {
        *self.bytes.borrow_mut() = Some(bytes.to_vec());
        Ok(())
    }
}

fn complete_state() -> ApplicationState {
    let mut state = ApplicationState::default();
    state.completed_lessons = BTreeSet::from(["lesson-001".to_owned()]);
    state.completed_islands = BTreeSet::from(["island-001".to_owned()]);
    state.current_lesson = Some("lesson-002".to_owned());
    state.sentence_states = BTreeMap::from([
        ("sentence-001".to_owned(), SentenceLearningState::Mastered),
        ("sentence-002".to_owned(), SentenceLearningState::Practice),
    ]);
    state.practice_pools =
        BTreeMap::from([("lesson-002".to_owned(), vec!["sentence-002".to_owned()])]);
    state.revision_records = BTreeMap::from([(
        "sentence-001".to_owned(),
        RevisionRecord {
            sentence_id: "sentence-001".to_owned(),
            last_revised: 1_900,
            revision_count: 2,
            status: RevisionStatus::Mastered,
            successful_revision_history: vec![1_000, 1_900],
        },
    )]);
    state.unfinished_lesson = Some(UnfinishedLesson {
        lesson_id: "lesson-002".to_owned(),
        next_introduction_position: 1,
        outcomes: BTreeMap::from([("sentence-002".to_owned(), RecallOutcome::Unknown)]),
        practice_order: vec!["sentence-002".to_owned()],
        practice_position: 0,
    });
    state.unfinished_revision_session = Some(UnfinishedRevisionSession {
        sentence_order: vec!["sentence-001".to_owned()],
        current_position: 0,
    });
    state.settings = PersistedSettings {
        default_revision_session_size: 15,
        mastery_threshold: 15,
        audio_volume_percent: 70,
    };
    state
}

#[test]
fn saves_and_restores_the_complete_application_state() {
    let persistence = Persistence::new(MemoryStore::default());
    let expected = complete_state();

    persistence.save(&expected).unwrap();
    let restored = persistence.load().unwrap();

    assert_eq!(restored, expected);
    assert_eq!(restored.version(), 1);
    assert_eq!(restored.revision_records["sentence-001"].revision_count, 2);
    assert_eq!(
        restored.revision_records["sentence-001"].successful_revision_history,
        [1_000, 1_900]
    );
}

#[test]
fn initializes_only_when_no_saved_state_exists() {
    let persistence = Persistence::new(MemoryStore::default());
    assert_eq!(persistence.load().unwrap(), ApplicationState::default());
}

#[test]
fn reopening_content_does_not_reset_saved_progress() {
    let store = MemoryStore::default();
    let persistence = Persistence::new(store);
    let expected = complete_state();
    persistence.save(&expected).unwrap();

    let first_open = persistence.load().unwrap();
    let second_open = persistence.load().unwrap();

    assert_eq!(first_open, expected);
    assert_eq!(second_open, expected);
}

#[test]
fn reports_corrupt_state_without_replacing_it() {
    let store = MemoryStore::with_bytes(b"not-json");
    let persistence = Persistence::new(store.clone());

    assert!(matches!(
        persistence.load(),
        Err(PersistenceError::InvalidState(_))
    ));
    assert_eq!(store.read().unwrap(), Some(b"not-json".to_vec()));
}

#[test]
fn reports_unsupported_state_without_resetting_it() {
    let bytes = br#"{"state_version":2}"#;
    let store = MemoryStore::with_bytes(bytes);
    let persistence = Persistence::new(store.clone());

    assert!(matches!(
        persistence.load(),
        Err(PersistenceError::UnsupportedVersion(2))
    ));
    assert_eq!(store.read().unwrap(), Some(bytes.to_vec()));
}

#[test]
fn rejects_invalid_state_before_saving() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    state.settings.audio_volume_percent = 101;

    assert!(matches!(
        persistence.save(&state),
        Err(PersistenceError::InvalidData(
            StateValidationError::InvalidAudioVolume(101)
        ))
    ));
}

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "korean-app-persistence-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn atomically_replaces_the_local_state_file() {
    let directory = TempDir::new();
    let path = directory.path().join("state/application.json");
    let persistence = Persistence::new(FileStateStore::new(&path));
    let mut state = complete_state();
    persistence.save(&state).unwrap();

    state.current_lesson = Some("lesson-003".to_owned());
    persistence.save(&state).unwrap();

    let reopened = Persistence::new(FileStateStore::new(&path));
    assert_eq!(reopened.load().unwrap(), state);
    assert!(path.exists());
    assert!(!path.with_file_name("application.json.tmp").exists());
}
