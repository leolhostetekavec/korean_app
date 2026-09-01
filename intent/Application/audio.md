---
tests_path: crates/app/tests
---

# Korean Audio

This component treats curated Korean audio generated with Google Cloud Text-to-Speech
Chirp 3: HD Korean voices as the pronunciation authority and coordinates local playback.

## V1 audio generation direction

Course audio is generated during content preparation, validated, and bundled with the
offline course. Runtime playback does not require a TTS network request. The generated
audio is course content; French-friendly pronunciation text is not an audio substitute.

## PlaybackContract

- binding: semantic
- artifact_path: crates/app/src/audio.rs

Audio playback must:

1. Resolve the Korean audio reference supplied for the current sentence.
2. Keep lit:Écouter disabled until the answer is revealed.
3. Play the same Korean recording whenever lit:Écouter is pressed repeatedly on one card.
4. Allow listening without changing the learner's answer, practice membership, revision count, or session position.
5. Work offline after the generated course audio has been bundled or downloaded.
6. Apply the supported audio preference without treating French-friendly pronunciation text as an audio substitute.
