use korean_app_core::persistence::{
    ApplicationState, Persistence, RevisionRecord, RevisionStatus, SentenceLearningState,
    StateStore, UnfinishedRevisionSession,
};
use korean_app_core::revision::{Revision, RevisionError, SessionProgress};
use korean_app_core::ui::RecallOutcome;
use std::cell::{Cell, RefCell};
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

fn record(id: &str, last_revised: u64, count: u32, status: RevisionStatus) -> RevisionRecord {
    RevisionRecord {
        sentence_id: id.to_owned(),
        last_revised,
        revision_count: count,
        status,
        successful_revision_history: Vec::new(),
    }
}

fn insert_record(state: &mut ApplicationState, record: RevisionRecord) {
    state
        .revision_records
        .insert(record.sentence_id.clone(), record);
}

#[test]
fn activates_a_sentence_once_without_replacing_its_history() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();

    {
        let mut revision = Revision::new(&mut state, &persistence);
        assert!(revision.activate_sentence("sentence-001", 100).unwrap());
        assert!(!revision.activate_sentence("sentence-001", 900).unwrap());
    }

    let record = &state.revision_records["sentence-001"];
    assert_eq!(record.last_revised, 100);
    assert_eq!(record.revision_count, 0);
    assert_eq!(record.status, RevisionStatus::Active);
    assert!(record.successful_revision_history.is_empty());
}

#[test]
fn prioritizes_unrevised_then_oldest_active_records() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    insert_record(
        &mut state,
        record("revised-old", 10, 2, RevisionStatus::Active),
    );
    insert_record(
        &mut state,
        record("new-recent", 40, 0, RevisionStatus::Active),
    );
    insert_record(&mut state, record("new-old", 20, 0, RevisionStatus::Active));
    insert_record(
        &mut state,
        record("revised-recent", 50, 1, RevisionStatus::Active),
    );
    insert_record(
        &mut state,
        record("mastered", 1, 0, RevisionStatus::Mastered),
    );

    let session = Revision::new(&mut state, &persistence)
        .request_session(3)
        .unwrap()
        .unwrap();

    assert_eq!(
        session.sentence_ids(),
        ["new-old", "new-recent", "revised-old"]
    );
    assert_eq!(session.current_sentence_id(), "new-old");
}

#[test]
fn uses_every_active_record_when_fewer_than_requested() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    insert_record(&mut state, record("a", 10, 0, RevisionStatus::Active));
    insert_record(&mut state, record("b", 20, 1, RevisionStatus::Active));

    let session = Revision::new(&mut state, &persistence)
        .request_session(20)
        .unwrap()
        .unwrap();

    assert_eq!(session.sentence_ids(), ["a", "b"]);
}

#[test]
fn restores_the_exact_unfinished_session_after_restart() {
    let store = MemoryStore::default();
    let persistence = Persistence::new(store.clone());
    let mut state = ApplicationState::default();
    insert_record(&mut state, record("a", 10, 0, RevisionStatus::Active));
    insert_record(&mut state, record("b", 20, 0, RevisionStatus::Active));

    {
        let mut revision = Revision::new(&mut state, &persistence);
        revision.request_session(2).unwrap();
        revision.assess(RecallOutcome::Unknown, 100).unwrap();
    }

    let mut restored = Persistence::new(store.clone()).load().unwrap();
    let reopened = Persistence::new(store);
    let session = Revision::new(&mut restored, &reopened)
        .request_session(1)
        .unwrap()
        .unwrap();

    assert_eq!(session.sentence_ids(), ["a", "b"]);
    assert_eq!(session.current_sentence_id(), "b");
}

#[test]
fn failed_sentences_cycle_behind_the_other_remaining_sentences() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    for (id, time) in [("a", 10), ("b", 20), ("c", 30)] {
        insert_record(&mut state, record(id, time, 0, RevisionStatus::Active));
    }
    {
        let mut revision = Revision::new(&mut state, &persistence);
        revision.request_session(3).unwrap();

        let progress = revision.assess(RecallOutcome::Unknown, 100).unwrap();
        let SessionProgress::InProgress(session) = progress else {
            panic!("session completed unexpectedly");
        };
        assert_eq!(session.current_sentence_id(), "b");

        let progress = revision.assess(RecallOutcome::Known, 110).unwrap();
        let SessionProgress::InProgress(session) = progress else {
            panic!("session completed unexpectedly");
        };
        assert_eq!(session.current_sentence_id(), "c");

        let progress = revision.assess(RecallOutcome::Known, 120).unwrap();
        let SessionProgress::InProgress(session) = progress else {
            panic!("session completed unexpectedly");
        };
        assert_eq!(session.current_sentence_id(), "a");
    }

    assert_eq!(state.revision_records["a"].last_revised, 10);
    assert_eq!(state.revision_records["a"].revision_count, 0);
    assert!(
        state.revision_records["a"]
            .successful_revision_history
            .is_empty()
    );
}

#[test]
fn success_updates_history_and_completes_the_session() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    insert_record(&mut state, record("a", 10, 0, RevisionStatus::Active));
    {
        let mut revision = Revision::new(&mut state, &persistence);
        revision.request_session(1).unwrap();

        assert_eq!(
            revision.assess(RecallOutcome::Known, 500).unwrap(),
            SessionProgress::Complete
        );
        assert!(revision.active_session().is_none());
    }

    let record = &state.revision_records["a"];
    assert_eq!(record.last_revised, 500);
    assert_eq!(record.revision_count, 1);
    assert_eq!(record.successful_revision_history, [500]);
}

#[test]
fn current_threshold_controls_mastery_without_removing_the_record() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    state.settings.mastery_threshold = 2;
    let mut existing = record("a", 10, 1, RevisionStatus::Active);
    existing.successful_revision_history.push(10);
    insert_record(&mut state, existing);
    {
        let mut revision = Revision::new(&mut state, &persistence);
        revision.request_session(1).unwrap();
        revision.assess(RecallOutcome::Known, 20).unwrap();
    }

    let record = &state.revision_records["a"];
    assert_eq!(record.status, RevisionStatus::Mastered);
    assert_eq!(record.revision_count, 2);
    assert_eq!(record.last_revised, 20);
    assert_eq!(record.successful_revision_history, [10, 20]);
    assert_eq!(state.sentence_states["a"], SentenceLearningState::Mastered);

    assert!(
        Revision::new(&mut state, &persistence)
            .request_session(10)
            .unwrap()
            .is_none()
    );
}

#[test]
fn allows_multiple_completed_sessions_on_the_same_day() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    insert_record(&mut state, record("a", 10, 0, RevisionStatus::Active));
    {
        let mut revision = Revision::new(&mut state, &persistence);
        revision.request_session(1).unwrap();
        revision.assess(RecallOutcome::Known, 1_000).unwrap();
        revision.request_session(1).unwrap();
        revision.assess(RecallOutcome::Known, 1_100).unwrap();
    }
    assert_eq!(state.revision_records["a"].revision_count, 2);
}

#[test]
fn validates_session_requests_and_outcomes() {
    let persistence = Persistence::new(MemoryStore::default());
    let mut state = ApplicationState::default();
    let mut revision = Revision::new(&mut state, &persistence);

    assert!(matches!(
        revision.request_session(0),
        Err(RevisionError::ZeroRequestedCount)
    ));
    assert!(matches!(
        revision.assess(RecallOutcome::Known, 10),
        Err(RevisionError::NoActiveSession)
    ));
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
fn rolls_back_state_when_a_session_cannot_be_persisted() {
    let persistence = Persistence::new(FailingStore::default());
    let mut state = ApplicationState::default();
    insert_record(&mut state, record("a", 10, 0, RevisionStatus::Active));
    let before = state.clone();

    let result = Revision::new(&mut state, &persistence).request_session(1);

    assert!(matches!(result, Err(RevisionError::Persistence(_))));
    assert_eq!(state, before);
}

#[test]
fn rolls_back_a_successful_assessment_when_it_cannot_be_persisted() {
    let persistence = Persistence::new(FailingStore::default());
    let mut state = ApplicationState::default();
    insert_record(&mut state, record("a", 10, 0, RevisionStatus::Active));
    state.unfinished_revision_session = Some(UnfinishedRevisionSession {
        sentence_order: vec!["a".to_owned()],
        current_position: 0,
    });
    let before = state.clone();

    let result = Revision::new(&mut state, &persistence).assess(RecallOutcome::Known, 100);

    assert!(matches!(result, Err(RevisionError::Persistence(_))));
    assert_eq!(state, before);
}
