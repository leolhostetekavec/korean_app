---
tests_path: crates/app/tests
---

# Korean Audio

This component treats native Korean audio as the pronunciation authority and coordinates local playback.

## PlaybackContract

- binding: semantic
- artifact_path: crates/app/src/audio.rs

Audio playback must:

1. Resolve the Korean audio reference supplied for the current sentence.
2. Keep lit:Écouter disabled until the answer is revealed.
3. Play the same Korean recording whenever lit:Écouter is pressed repeatedly on one card.
4. Allow listening without changing the learner's answer, practice membership, revision count, or session position.
5. Work offline after the course audio has been bundled or downloaded.
6. Apply the supported audio preference without treating generated pronunciation text as an audio substitute.
