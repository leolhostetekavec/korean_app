use crate::audio::{AudioPreference, PreferenceError};
use crate::persistence::{ApplicationState, Persistence, PersistenceError, StateStore};
use std::error::Error;
use std::fmt;

/// Validated access to learner settings backed by the application's local state.
pub struct LearnerSettings<'a, S> {
    state: &'a mut ApplicationState,
    persistence: &'a Persistence<S>,
}

impl<'a, S> LearnerSettings<'a, S>
where
    S: StateStore,
{
    pub fn new(state: &'a mut ApplicationState, persistence: &'a Persistence<S>) -> Self {
        Self { state, persistence }
    }

    pub fn default_revision_session_size(&self) -> usize {
        self.state.settings.default_revision_session_size
    }

    pub fn mastery_threshold(&self) -> u32 {
        self.state.settings.mastery_threshold
    }

    pub fn audio_preference(&self) -> Result<AudioPreference, PreferenceError> {
        AudioPreference::new(self.state.settings.audio_volume_percent)
    }

    pub fn set_default_revision_session_size(
        &mut self,
        size: usize,
    ) -> Result<(), SettingsError<S::Error>> {
        if size == 0 {
            return Err(SettingsError::ZeroRevisionSessionSize);
        }
        self.persist(|state| state.settings.default_revision_session_size = size)
    }

    pub fn set_mastery_threshold(&mut self, threshold: u32) -> Result<(), SettingsError<S::Error>> {
        if threshold == 0 {
            return Err(SettingsError::ZeroMasteryThreshold);
        }
        self.persist(|state| state.settings.mastery_threshold = threshold)
    }

    pub fn set_audio_volume(&mut self, volume_percent: u8) -> Result<(), SettingsError<S::Error>> {
        AudioPreference::new(volume_percent).map_err(SettingsError::InvalidAudioVolume)?;
        self.persist(|state| state.settings.audio_volume_percent = volume_percent)
    }

    fn persist(
        &mut self,
        update: impl FnOnce(&mut ApplicationState),
    ) -> Result<(), SettingsError<S::Error>> {
        let previous = self.state.settings.clone();
        update(self.state);

        if let Err(error) = self.persistence.save(self.state) {
            self.state.settings = previous;
            return Err(SettingsError::Persistence(error));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum SettingsError<E> {
    ZeroRevisionSessionSize,
    ZeroMasteryThreshold,
    InvalidAudioVolume(PreferenceError),
    Persistence(PersistenceError<E>),
}

impl<E> fmt::Display for SettingsError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRevisionSessionSize => {
                formatter.write_str("default revision session size must be greater than zero")
            }
            Self::ZeroMasteryThreshold => {
                formatter.write_str("mastery threshold must be greater than zero")
            }
            Self::InvalidAudioVolume(error) => error.fmt(formatter),
            Self::Persistence(error) => error.fmt(formatter),
        }
    }
}

impl<E> Error for SettingsError<E> where E: Error + 'static {}
