use korean_app_core::content::{
    AudioAssetCatalog, CourseBundle, ImportError, SUPPORTED_BUNDLE_VERSION,
};
use std::collections::HashSet;

struct Assets(HashSet<String>);

impl Assets {
    fn containing(references: &[&str]) -> Self {
        Self(
            references
                .iter()
                .map(|reference| (*reference).to_owned())
                .collect(),
        )
    }
}

impl AudioAssetCatalog for Assets {
    fn contains(&self, audio_ref: &str) -> bool {
        self.0.contains(audio_ref)
    }
}

const VALID_BUNDLE: &str = r#"
{
  "bundle_version": 1,
  "tts": {
    "provider": "Google Cloud Text-to-Speech",
    "model": "Chirp 3: HD"
  },
  "objectives": ["Saluer poliment"],
  "islands": [
    {
      "island_id": "island-greetings",
      "objectives": ["Commencer une conversation"],
      "lessons": [
        {
          "lesson_id": "lesson-hello",
          "objectives": ["Dire bonjour"],
          "sentence_order": ["sentence-002", "sentence-001"],
          "sentences": {
            "sentence-001": {
              "french_prompt": "Bonjour",
              "korean_text": "안녕하세요",
              "french_pronunciation": "ann-yong-ha-se-yo",
              "audio_ref": "audio/sentence-001.mp3"
            },
            "sentence-002": {
              "french_prompt": "Merci",
              "korean_text": "감사합니다",
              "french_pronunciation": "kam-sa-ham-ni-da",
              "audio_ref": "audio/sentence-002.mp3"
            }
          },
          "curriculum_note": "Unknown curriculum metadata remains LanguageContent-owned."
        }
      ]
    }
  ]
}
"#;

fn valid_assets() -> Assets {
    Assets::containing(&["audio/sentence-001.mp3", "audio/sentence-002.mp3"])
}

#[test]
fn imports_content_without_rewriting_it() {
    let bundle = CourseBundle::import_json(VALID_BUNDLE, &valid_assets()).unwrap();

    assert_eq!(bundle.version(), SUPPORTED_BUNDLE_VERSION);
    assert_eq!(bundle.objectives(), ["Saluer poliment"]);
    let island = &bundle.islands()[0];
    assert_eq!(island.id(), "island-greetings");
    let lesson = &island.lessons()[0];
    assert_eq!(lesson.id(), "lesson-hello");
    assert_eq!(
        lesson
            .sentences()
            .iter()
            .map(|sentence| sentence.id())
            .collect::<Vec<_>>(),
        ["sentence-002", "sentence-001"]
    );
    let sentence = &lesson.sentences()[1];
    assert_eq!(sentence.french_prompt(), "Bonjour");
    assert_eq!(sentence.korean_text(), "안녕하세요");
    assert_eq!(sentence.french_pronunciation(), "ann-yong-ha-se-yo");
    assert_eq!(sentence.audio_ref(), "audio/sentence-001.mp3");
}

#[test]
fn rejects_an_unsupported_bundle_version() {
    let json = VALID_BUNDLE.replace("\"bundle_version\": 1", "\"bundle_version\": 2");
    assert_eq!(
        CourseBundle::import_json(&json, &valid_assets()),
        Err(ImportError::UnsupportedVersion(2))
    );
}

#[test]
fn rejects_a_non_chirp_tts_source() {
    let json = VALID_BUNDLE.replace("Chirp 3: HD", "Other model");
    assert!(matches!(
        CourseBundle::import_json(&json, &valid_assets()),
        Err(ImportError::InvalidTtsSource { .. })
    ));
}

#[test]
fn rejects_duplicate_sentence_keys() {
    let json = VALID_BUNDLE.replace("\"sentence-002\": {", "\"sentence-001\": {");
    assert!(matches!(
        CourseBundle::import_json(&json, &valid_assets()),
        Err(ImportError::InvalidJson(message)) if message.contains("duplicate sentence identity")
    ));
}

#[test]
fn rejects_a_sentence_identity_reused_in_another_lesson() {
    let second_lesson = r#",
        {
          "lesson_id": "lesson-second",
          "sentence_order": ["sentence-001"],
          "sentences": {
            "sentence-001": {
              "french_prompt": "Salut",
              "korean_text": "안녕",
              "french_pronunciation": "ann-yong",
              "audio_ref": "audio/sentence-001.mp3"
            }
          }
        }"#;
    let json = VALID_BUNDLE.replace(
        "      ]\n    }",
        &format!("{second_lesson}\n      ]\n    }}"),
    );
    assert_eq!(
        CourseBundle::import_json(&json, &valid_assets()),
        Err(ImportError::DuplicateIdentity {
            kind: "sentence",
            id: "sentence-001".to_owned(),
        })
    );
}

#[test]
fn rejects_missing_required_sentence_content() {
    let json = VALID_BUNDLE.replace("\"french_prompt\": \"Bonjour\",", "");
    assert!(matches!(
        CourseBundle::import_json(&json, &valid_assets()),
        Err(ImportError::InvalidJson(message)) if message.contains("french_prompt")
    ));
}

#[test]
fn rejects_unknown_and_unordered_sentence_identities() {
    let unknown = VALID_BUNDLE.replace(
        "[\"sentence-002\", \"sentence-001\"]",
        "[\"sentence-404\", \"sentence-001\"]",
    );
    assert!(matches!(
        CourseBundle::import_json(&unknown, &valid_assets()),
        Err(ImportError::UnknownOrderIdentity { sentence_id, .. }) if sentence_id == "sentence-404"
    ));

    let missing =
        VALID_BUNDLE.replace("[\"sentence-002\", \"sentence-001\"]", "[\"sentence-001\"]");
    assert!(matches!(
        CourseBundle::import_json(&missing, &valid_assets()),
        Err(ImportError::MissingOrderIdentity { sentence_id, .. }) if sentence_id == "sentence-002"
    ));
}

#[test]
fn rejects_duplicate_sentence_order_entries() {
    let json = VALID_BUNDLE.replace(
        "[\"sentence-002\", \"sentence-001\"]",
        "[\"sentence-001\", \"sentence-001\"]",
    );
    assert!(matches!(
        CourseBundle::import_json(&json, &valid_assets()),
        Err(ImportError::DuplicateOrderIdentity { sentence_id, .. }) if sentence_id == "sentence-001"
    ));
}

#[test]
fn rejects_missing_and_unsafe_audio_assets() {
    assert!(matches!(
        CourseBundle::import_json(VALID_BUNDLE, &Assets::containing(&[])),
        Err(ImportError::MissingAudioAsset { .. })
    ));

    let unsafe_ref = VALID_BUNDLE.replace(
        "audio/sentence-001.mp3",
        "audio/../private/sentence-001.mp3",
    );
    assert!(matches!(
        CourseBundle::import_json(&unsafe_ref, &valid_assets()),
        Err(ImportError::InvalidAudioReference { .. })
    ));
}
