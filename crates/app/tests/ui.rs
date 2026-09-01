use korean_app_core::audio::{AudioPreference, LocalAudioPlayer, PlaybackError};
use korean_app_core::content::{AudioAssetCatalog, CourseBundle, Sentence};
use korean_app_core::ui::{
    AssessmentError, KNOWN_ACTION, LISTEN_ACTION, REVEAL_ACTION, RecallCard, RecallOutcome,
    UNKNOWN_ACTION,
};
use std::convert::Infallible;

struct Assets;

impl AudioAssetCatalog for Assets {
    fn contains(&self, audio_ref: &str) -> bool {
        audio_ref == "audio/sentence-001.mp3"
    }
}

#[derive(Default)]
struct Player(Vec<String>);

impl LocalAudioPlayer for Player {
    type Error = Infallible;

    fn play_local(
        &mut self,
        audio_ref: &str,
        _preference: AudioPreference,
    ) -> Result<(), Self::Error> {
        self.0.push(audio_ref.to_owned());
        Ok(())
    }
}

fn sentence() -> Sentence {
    let json = r#"
    {
      "bundle_version": 1,
      "tts": {
        "provider": "Google Cloud Text-to-Speech",
        "model": "Chirp 3: HD"
      },
      "islands": [
        {
          "island_id": "island-001",
          "lessons": [
            {
              "lesson_id": "lesson-001",
              "sentence_order": ["sentence-001"],
              "sentences": {
                "sentence-001": {
                  "french_prompt": "Bonjour",
                  "korean_text": "안녕하세요",
                  "french_pronunciation": "ann-yong-ha-se-yo",
                  "audio_ref": "audio/sentence-001.mp3"
                }
              }
            }
          ]
        }
      ]
    }
    "#;
    CourseBundle::import_json(json, &Assets).unwrap().islands()[0].lessons()[0].sentences()[0]
        .clone()
}

#[test]
fn initially_projects_only_the_french_prompt_and_reveal_action() {
    let sentence = sentence();
    let card = RecallCard::new(&sentence, AudioPreference::default());
    let projection = card.projection();

    assert_eq!(projection.french_prompt(), "Bonjour");
    assert_eq!(projection.reveal_action(), Some(REVEAL_ACTION));
    assert_eq!(projection.french_pronunciation(), None);
    assert_eq!(projection.listen_action(), None);
    assert_eq!(projection.known_action(), None);
    assert_eq!(projection.unknown_action(), None);
    assert_ne!(projection.french_prompt(), sentence.korean_text());
}

#[test]
fn reveals_only_the_pronunciation_and_exact_actions() {
    let sentence = sentence();
    let mut card = RecallCard::new(&sentence, AudioPreference::default());
    card.reveal_answer();
    let projection = card.projection();

    assert_eq!(projection.french_pronunciation(), Some("ann-yong-ha-se-yo"));
    assert_eq!(projection.reveal_action(), None);
    assert_eq!(projection.listen_action(), Some(LISTEN_ACTION));
    assert_eq!(projection.known_action(), Some(KNOWN_ACTION));
    assert_eq!(projection.unknown_action(), Some(UNKNOWN_ACTION));
    assert_ne!(
        projection.french_pronunciation(),
        Some(sentence.korean_text())
    );
}

#[test]
fn blocks_audio_and_outcomes_before_reveal() {
    let sentence = sentence();
    let card = RecallCard::new(&sentence, AudioPreference::default());
    let mut player = Player::default();

    assert_eq!(card.listen(&mut player), Err(PlaybackError::AnswerHidden));
    assert_eq!(
        card.assess(RecallOutcome::Known),
        Err(AssessmentError::AnswerHidden)
    );
    assert!(player.0.is_empty());
}

#[test]
fn dispatches_the_sentence_audio_after_reveal() {
    let sentence = sentence();
    let mut card = RecallCard::new(&sentence, AudioPreference::default());
    let mut player = Player::default();
    card.reveal_answer();

    card.listen(&mut player).unwrap();
    assert_eq!(player.0, ["audio/sentence-001.mp3"]);
}

#[test]
fn exposes_both_self_assessment_outcomes_after_reveal() {
    let sentence = sentence();
    let mut card = RecallCard::new(&sentence, AudioPreference::default());
    card.reveal_answer();

    assert_eq!(card.assess(RecallOutcome::Known), Ok(RecallOutcome::Known));
    assert_eq!(
        card.assess(RecallOutcome::Unknown),
        Ok(RecallOutcome::Unknown)
    );
}
