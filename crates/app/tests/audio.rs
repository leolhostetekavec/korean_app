use korean_app_core::audio::{AudioPreference, CardAudio, LocalAudioPlayer, PlaybackError};
use korean_app_core::content::{AudioAssetCatalog, CourseBundle, Sentence};
use std::error::Error;
use std::fmt;

struct Assets;

impl AudioAssetCatalog for Assets {
    fn contains(&self, audio_ref: &str) -> bool {
        audio_ref == "audio/sentence-001.mp3"
    }
}

#[derive(Debug, Default)]
struct Player {
    calls: Vec<(String, AudioPreference)>,
    fail: bool,
}

impl LocalAudioPlayer for Player {
    type Error = PlayerError;

    fn play_local(
        &mut self,
        audio_ref: &str,
        preference: AudioPreference,
    ) -> Result<(), Self::Error> {
        self.calls.push((audio_ref.to_owned(), preference));
        if self.fail {
            return Err(PlayerError);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlayerError;

impl fmt::Display for PlayerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("device rejected the asset")
    }
}

impl Error for PlayerError {}

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
fn keeps_listening_disabled_before_reveal() {
    let sentence = sentence();
    let audio = CardAudio::new(&sentence, AudioPreference::default());
    let mut player = Player::default();

    assert!(!audio.can_listen());
    assert_eq!(audio.listen(&mut player), Err(PlaybackError::AnswerHidden));
    assert!(player.calls.is_empty());
}

#[test]
fn repeatedly_plays_the_same_local_course_asset() {
    let sentence = sentence();
    let mut audio = CardAudio::new(&sentence, AudioPreference::default());
    let mut player = Player::default();
    audio.reveal_answer();

    audio.listen(&mut player).unwrap();
    audio.listen(&mut player).unwrap();

    assert!(audio.can_listen());
    assert_eq!(player.calls.len(), 2);
    assert!(
        player
            .calls
            .iter()
            .all(|(reference, _)| reference == "audio/sentence-001.mp3")
    );
}

#[test]
fn applies_preference_updates_to_subsequent_playback() {
    let sentence = sentence();
    let mut audio = CardAudio::new(&sentence, AudioPreference::new(80).unwrap());
    let mut player = Player::default();
    audio.reveal_answer();
    audio.listen(&mut player).unwrap();

    audio.set_preference(AudioPreference::new(35).unwrap());
    audio.listen(&mut player).unwrap();

    assert_eq!(player.calls[0].1.volume_percent(), 80);
    assert_eq!(player.calls[1].1.volume_percent(), 35);
    assert_eq!(audio.preference().volume_percent(), 35);
}

#[test]
fn rejects_an_invalid_volume() {
    let error = AudioPreference::new(101).unwrap_err();
    assert_eq!(error.volume_percent(), 101);
}

#[test]
fn reports_local_player_failures() {
    let sentence = sentence();
    let mut audio = CardAudio::new(&sentence, AudioPreference::default());
    let mut player = Player {
        fail: true,
        ..Player::default()
    };
    audio.reveal_answer();

    assert_eq!(
        audio.listen(&mut player),
        Err(PlaybackError::Player(PlayerError))
    );
}

#[test]
fn playback_does_not_modify_sentence_content() {
    let sentence = sentence();
    let original = sentence.clone();
    let mut audio = CardAudio::new(&sentence, AudioPreference::default());
    let mut player = Player::default();
    audio.reveal_answer();
    audio.listen(&mut player).unwrap();

    assert_eq!(sentence, original);
}
