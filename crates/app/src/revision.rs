use crate::persistence::{
    ApplicationState, Persistence, PersistenceError, RevisionRecord, RevisionStatus,
    SentenceLearningState, StateStore, UnfinishedRevisionSession,
};
use crate::ui::RecallOutcome;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionSession {
    sentence_ids: Vec<String>,
    current_position: usize,
}

impl RevisionSession {
    pub fn sentence_ids(&self) -> &[String] {
        &self.sentence_ids
    }

    pub fn current_sentence_id(&self) -> &str {
        &self.sentence_ids[self.current_position]
    }
}

impl From<&UnfinishedRevisionSession> for RevisionSession {
    fn from(session: &UnfinishedRevisionSession) -> Self {
        Self {
            sentence_ids: session.sentence_order.clone(),
            current_position: session.current_position,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionProgress {
    InProgress(RevisionSession),
    Complete,
}

/// Coordinates learner-initiated revision against locally persisted application state.
pub struct Revision<'a, S> {
    state: &'a mut ApplicationState,
    persistence: &'a Persistence<S>,
}

impl<'a, S> Revision<'a, S>
where
    S: StateStore,
{
    pub fn new(state: &'a mut ApplicationState, persistence: &'a Persistence<S>) -> Self {
        Self { state, persistence }
    }

    pub fn activate_sentence(
        &mut self,
        sentence_id: impl Into<String>,
        activated_at: u64,
    ) -> Result<bool, RevisionError<S::Error>> {
        let sentence_id = sentence_id.into();
        if sentence_id.trim().is_empty() {
            return Err(RevisionError::EmptySentenceId);
        }
        if self.state.revision_records.contains_key(&sentence_id) {
            return Ok(false);
        }

        let previous = self.state.clone();
        self.state.revision_records.insert(
            sentence_id.clone(),
            RevisionRecord {
                sentence_id: sentence_id.clone(),
                last_revised: activated_at,
                revision_count: 0,
                status: RevisionStatus::Active,
                successful_revision_history: Vec::new(),
            },
        );
        self.state
            .sentence_states
            .insert(sentence_id, SentenceLearningState::Active);
        self.save_or_restore(previous)?;
        Ok(true)
    }

    pub fn request_session(
        &mut self,
        requested_count: usize,
    ) -> Result<Option<RevisionSession>, RevisionError<S::Error>> {
        if let Some(session) = &self.state.unfinished_revision_session {
            return Ok(Some(RevisionSession::from(session)));
        }
        if requested_count == 0 {
            return Err(RevisionError::ZeroRequestedCount);
        }

        let mut candidates: Vec<_> = self
            .state
            .revision_records
            .values()
            .filter(|record| record.status == RevisionStatus::Active)
            .collect();
        candidates.sort_by(|left, right| {
            (
                left.revision_count != 0,
                left.last_revised,
                &left.sentence_id,
            )
                .cmp(&(
                    right.revision_count != 0,
                    right.last_revised,
                    &right.sentence_id,
                ))
        });

        let sentence_order: Vec<_> = candidates
            .into_iter()
            .take(requested_count)
            .map(|record| record.sentence_id.clone())
            .collect();
        if sentence_order.is_empty() {
            return Ok(None);
        }

        let previous = self.state.clone();
        self.state.unfinished_revision_session = Some(UnfinishedRevisionSession {
            sentence_order,
            current_position: 0,
        });
        self.save_or_restore(previous)?;
        Ok(self.active_session())
    }

    pub fn request_default_session(
        &mut self,
    ) -> Result<Option<RevisionSession>, RevisionError<S::Error>> {
        self.request_session(self.state.settings.default_revision_session_size)
    }

    pub fn active_session(&self) -> Option<RevisionSession> {
        self.state
            .unfinished_revision_session
            .as_ref()
            .map(RevisionSession::from)
    }

    pub fn assess(
        &mut self,
        outcome: RecallOutcome,
        revised_at: u64,
    ) -> Result<SessionProgress, RevisionError<S::Error>> {
        let Some(session) = &self.state.unfinished_revision_session else {
            return Err(RevisionError::NoActiveSession);
        };
        let sentence_id = session.sentence_order[session.current_position].clone();
        let previous = self.state.clone();

        let result = match outcome {
            RecallOutcome::Known => self.record_success(&sentence_id, revised_at),
            RecallOutcome::Unknown => {
                let session = self
                    .state
                    .unfinished_revision_session
                    .as_mut()
                    .expect("active session was checked above");
                session.current_position =
                    (session.current_position + 1) % session.sentence_order.len();
                Ok(())
            }
        };
        if let Err(error) = result {
            *self.state = previous;
            return Err(error);
        }

        self.save_or_restore(previous)?;
        Ok(match self.active_session() {
            Some(session) => SessionProgress::InProgress(session),
            None => SessionProgress::Complete,
        })
    }

    fn record_success(
        &mut self,
        sentence_id: &str,
        revised_at: u64,
    ) -> Result<(), RevisionError<S::Error>> {
        let record = self
            .state
            .revision_records
            .get_mut(sentence_id)
            .ok_or_else(|| RevisionError::MissingRevisionRecord(sentence_id.to_owned()))?;
        record.revision_count = record
            .revision_count
            .checked_add(1)
            .ok_or_else(|| RevisionError::RevisionCountOverflow(sentence_id.to_owned()))?;
        record.last_revised = revised_at;
        record.successful_revision_history.push(revised_at);

        if record.revision_count >= self.state.settings.mastery_threshold {
            record.status = RevisionStatus::Mastered;
            self.state
                .sentence_states
                .insert(sentence_id.to_owned(), SentenceLearningState::Mastered);
        }

        let session = self
            .state
            .unfinished_revision_session
            .as_mut()
            .expect("active session was checked before recording success");
        session.sentence_order.remove(session.current_position);
        if session.sentence_order.is_empty() {
            self.state.unfinished_revision_session = None;
        } else {
            session.current_position %= session.sentence_order.len();
        }
        Ok(())
    }

    fn save_or_restore(
        &mut self,
        previous: ApplicationState,
    ) -> Result<(), RevisionError<S::Error>> {
        if let Err(error) = self.persistence.save(self.state) {
            *self.state = previous;
            return Err(RevisionError::Persistence(error));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RevisionError<E> {
    EmptySentenceId,
    ZeroRequestedCount,
    NoActiveSession,
    MissingRevisionRecord(String),
    RevisionCountOverflow(String),
    Persistence(PersistenceError<E>),
}

impl<E> fmt::Display for RevisionError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySentenceId => formatter.write_str("sentence identity must not be empty"),
            Self::ZeroRequestedCount => {
                formatter.write_str("revision session size must be greater than zero")
            }
            Self::NoActiveSession => formatter.write_str("there is no active revision session"),
            Self::MissingRevisionRecord(sentence_id) => {
                write!(formatter, "revision record {sentence_id:?} does not exist")
            }
            Self::RevisionCountOverflow(sentence_id) => {
                write!(formatter, "revision count for {sentence_id:?} overflowed")
            }
            Self::Persistence(error) => error.fmt(formatter),
        }
    }
}

impl<E> Error for RevisionError<E> where E: Error + 'static {}
