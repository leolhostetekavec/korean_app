use korean_app_core::persistence::{
    ApplicationState, Persistence, SentenceLearningState, StateStore,
};
use korean_app_core::settings::{LearnerSettings, SettingsError};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Default)]
struct MemoryStore {
    bytes: Rc<RefCell<Option<Vec<u8>>>>,
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

#[test]
fn exposes_safe_defaults_for_subsequent_behavior() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    let settings = LearnerSettings::new(&mut state, &persistence);

    assert_eq!(settings.default_revision_session_size(), 10);
    assert_eq!(settings.mastery_threshold(), 10);
    assert_eq!(settings.audio_preference().unwrap().volume_percent(), 100);
}

#[test]
fn persists_each_change_and_preserves_learning_history() {
    let store = MemoryStore::default();
    let persistence = Persistence::new(store.clone());
    let mut state = ApplicationState::default();
    state.current_lesson = Some("lesson-002".to_owned());
    state.sentence_states =
        BTreeMap::from([("sentence-001".to_owned(), SentenceLearningState::Active)]);
    let history = state.sentence_states.clone();

    {
        let mut settings = LearnerSettings::new(&mut state, &persistence);
        settings.set_default_revision_session_size(15).unwrap();
        settings.set_mastery_threshold(15).unwrap();
        settings.set_audio_volume(65).unwrap();

        assert_eq!(settings.default_revision_session_size(), 15);
        assert_eq!(settings.mastery_threshold(), 15);
        assert_eq!(settings.audio_preference().unwrap().volume_percent(), 65);
    }

    assert_eq!(state.sentence_states, history);
    assert_eq!(state.current_lesson.as_deref(), Some("lesson-002"));

    let restored = Persistence::new(store).load().unwrap();
    assert_eq!(restored.settings.default_revision_session_size, 15);
    assert_eq!(restored.settings.mastery_threshold, 15);
    assert_eq!(restored.settings.audio_volume_percent, 65);
    assert_eq!(restored.sentence_states, history);
}

#[test]
fn rejects_invalid_values_without_changing_or_saving_state() {
    let store = MemoryStore::default();
    let persistence = Persistence::new(store.clone());
    let mut state = ApplicationState::default();
    let mut settings = LearnerSettings::new(&mut state, &persistence);

    assert!(matches!(
        settings.set_default_revision_session_size(0),
        Err(SettingsError::ZeroRevisionSessionSize)
    ));
    assert!(matches!(
        settings.set_mastery_threshold(0),
        Err(SettingsError::ZeroMasteryThreshold)
    ));
    assert!(matches!(
        settings.set_audio_volume(101),
        Err(SettingsError::InvalidAudioVolume(_))
    ));

    assert_eq!(settings.default_revision_session_size(), 10);
    assert_eq!(settings.mastery_threshold(), 10);
    assert_eq!(settings.audio_preference().unwrap().volume_percent(), 100);
    assert!(store.read().unwrap().is_none());
}

#[derive(Clone, Copy, Debug)]
struct WriteFailed;

impl fmt::Display for WriteFailed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("write failed")
    }
}

impl Error for WriteFailed {}

#[derive(Default)]
struct FailingStore {
    writes: Cell<usize>,
}

impl StateStore for FailingStore {
    type Error = WriteFailed;

    fn read(&self) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }

    fn write(&self, _bytes: &[u8]) -> Result<(), Self::Error> {
        self.writes.set(self.writes.get() + 1);
        Err(WriteFailed)
    }
}

#[test]
fn rolls_back_in_memory_settings_when_persistence_fails() {
    let persistence = Persistence::new(FailingStore::default());
    let mut state = ApplicationState::default();
    let mut settings = LearnerSettings::new(&mut state, &persistence);

    assert!(matches!(
        settings.set_mastery_threshold(15),
        Err(SettingsError::Persistence(_))
    ));
    assert_eq!(settings.mastery_threshold(), 10);
}
