use crate::content::Sentence;
use std::error::Error;
use std::fmt;

/// Learner-controlled settings applied to each local playback request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioPreference {
    volume_percent: u8,
}

impl AudioPreference {
    pub fn new(volume_percent: u8) -> Result<Self, PreferenceError> {
        if volume_percent > 100 {
            return Err(PreferenceError(volume_percent));
        }
        Ok(Self { volume_percent })
    }

    pub fn volume_percent(self) -> u8 {
        self.volume_percent
    }
}

impl Default for AudioPreference {
    fn default() -> Self {
        Self {
            volume_percent: 100,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreferenceError(u8);

impl PreferenceError {
    pub fn volume_percent(self) -> u8 {
        self.0
    }
}

impl fmt::Display for PreferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "audio volume {} is outside 0..=100", self.0)
    }
}

impl Error for PreferenceError {}

/// Platform boundary for playing an already available course asset.
pub trait LocalAudioPlayer {
    type Error;

    fn play_local(
        &mut self,
        audio_ref: &str,
        preference: AudioPreference,
    ) -> Result<(), Self::Error>;
}

/// Audio state for one recall card.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardAudio<'a> {
    audio_ref: &'a str,
    answer_revealed: bool,
    preference: AudioPreference,
}

impl<'a> CardAudio<'a> {
    pub fn new(sentence: &'a Sentence, preference: AudioPreference) -> Self {
        Self {
            audio_ref: sentence.audio_ref(),
            answer_revealed: false,
            preference,
        }
    }

    pub fn reveal_answer(&mut self) {
        self.answer_revealed = true;
    }

    pub fn can_listen(&self) -> bool {
        self.answer_revealed
    }

    pub fn preference(&self) -> AudioPreference {
        self.preference
    }

    pub fn set_preference(&mut self, preference: AudioPreference) {
        self.preference = preference;
    }

    pub fn listen<P>(&self, player: &mut P) -> Result<(), PlaybackError<P::Error>>
    where
        P: LocalAudioPlayer,
    {
        if !self.answer_revealed {
            return Err(PlaybackError::AnswerHidden);
        }
        player
            .play_local(self.audio_ref, self.preference)
            .map_err(PlaybackError::Player)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaybackError<E> {
    AnswerHidden,
    Player(E),
}

impl<E> fmt::Display for PlaybackError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AnswerHidden => {
                formatter.write_str("audio is disabled until the answer is revealed")
            }
            Self::Player(error) => write!(formatter, "local audio playback failed: {error}"),
        }
    }
}

impl<E> Error for PlaybackError<E> where E: Error + 'static {}
