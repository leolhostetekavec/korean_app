use crate::audio::{AudioPreference, CardAudio, LocalAudioPlayer, PlaybackError};
use crate::content::Sentence;
use std::error::Error;
use std::fmt;

pub const REVEAL_ACTION: &str = "Voir la réponse";
pub const LISTEN_ACTION: &str = "Écouter";
pub const KNOWN_ACTION: &str = "Je connais";
pub const UNKNOWN_ACTION: &str = "Je ne connais pas";

slint::slint! {
    component ActionButton inherits Rectangle {
        in property <string> label;
        callback activated();

        height: 48px;
        border-radius: 8px;
        background: #2457c5;

        Text {
            text: root.label;
            color: white;
            horizontal-alignment: center;
            vertical-alignment: center;
        }

        TouchArea {
            clicked => { root.activated(); }
        }
    }

    export component RecallCardView inherits Window {
        in property <string> french-prompt;
        in property <string> french-pronunciation;
        in-out property <bool> answer-revealed: false;

        callback reveal-answer();
        callback listen();
        callback assess-known();
        callback assess-unknown();

        background: white;

        VerticalLayout {
            spacing: 12px;
            padding: 20px;

            Text {
                text: root.french-prompt;
                color: #161616;
                horizontal-alignment: center;
                wrap: word-wrap;
            }

            if !root.answer-revealed: ActionButton {
                label: "Voir la réponse";
                activated => {
                    root.answer-revealed = true;
                    root.reveal-answer();
                }
            }

            if root.answer-revealed: Text {
                text: root.french-pronunciation;
                color: #161616;
                horizontal-alignment: center;
                wrap: word-wrap;
            }

            if root.answer-revealed: ActionButton {
                label: "Écouter";
                activated => { root.listen(); }
            }

            if root.answer-revealed: HorizontalLayout {
                spacing: 12px;

                ActionButton {
                    label: "Je connais";
                    activated => { root.assess-known(); }
                }

                ActionButton {
                    label: "Je ne connais pas";
                    activated => { root.assess-unknown(); }
                }
            }
        }
    }
}

/// Contract-facing state for one lesson, practice, or revision card.
pub struct RecallCard<'a> {
    sentence: &'a Sentence,
    audio: CardAudio<'a>,
    answer_revealed: bool,
}

impl<'a> RecallCard<'a> {
    pub fn new(sentence: &'a Sentence, audio_preference: AudioPreference) -> Self {
        Self {
            sentence,
            audio: CardAudio::new(sentence, audio_preference),
            answer_revealed: false,
        }
    }

    pub fn projection(&self) -> RecallCardProjection<'_> {
        RecallCardProjection {
            french_prompt: self.sentence.french_prompt(),
            french_pronunciation: self
                .answer_revealed
                .then(|| self.sentence.french_pronunciation()),
            reveal_action: (!self.answer_revealed).then_some(REVEAL_ACTION),
            listen_action: self.answer_revealed.then_some(LISTEN_ACTION),
            known_action: self.answer_revealed.then_some(KNOWN_ACTION),
            unknown_action: self.answer_revealed.then_some(UNKNOWN_ACTION),
        }
    }

    pub fn reveal_answer(&mut self) {
        self.answer_revealed = true;
        self.audio.reveal_answer();
    }

    pub fn listen<P>(&self, player: &mut P) -> Result<(), PlaybackError<P::Error>>
    where
        P: LocalAudioPlayer,
    {
        self.audio.listen(player)
    }

    pub fn assess(&self, outcome: RecallOutcome) -> Result<RecallOutcome, AssessmentError> {
        if !self.answer_revealed {
            return Err(AssessmentError::AnswerHidden);
        }
        Ok(outcome)
    }

    pub fn set_audio_preference(&mut self, preference: AudioPreference) {
        self.audio.set_preference(preference);
    }

    pub fn apply_to_view(&self, view: &RecallCardView) {
        let projection = self.projection();
        view.set_french_prompt(projection.french_prompt().into());
        view.set_french_pronunciation(projection.french_pronunciation().unwrap_or_default().into());
        view.set_answer_revealed(self.answer_revealed);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecallCardProjection<'a> {
    french_prompt: &'a str,
    french_pronunciation: Option<&'a str>,
    reveal_action: Option<&'static str>,
    listen_action: Option<&'static str>,
    known_action: Option<&'static str>,
    unknown_action: Option<&'static str>,
}

impl<'a> RecallCardProjection<'a> {
    pub fn french_prompt(self) -> &'a str {
        self.french_prompt
    }

    pub fn french_pronunciation(self) -> Option<&'a str> {
        self.french_pronunciation
    }

    pub fn reveal_action(self) -> Option<&'static str> {
        self.reveal_action
    }

    pub fn listen_action(self) -> Option<&'static str> {
        self.listen_action
    }

    pub fn known_action(self) -> Option<&'static str> {
        self.known_action
    }

    pub fn unknown_action(self) -> Option<&'static str> {
        self.unknown_action
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecallOutcome {
    Known,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssessmentError {
    AnswerHidden,
}

impl fmt::Display for AssessmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AnswerHidden => {
                formatter.write_str("self-assessment is disabled until the answer is revealed")
            }
        }
    }
}

impl Error for AssessmentError {}
