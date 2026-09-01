---
tests_path: crates/app/tests
---

# Learner Settings

This component holds the small set of learner-adjustable V1 behaviors.

## SettingsContract

- binding: semantic
- artifact_path: crates/app/src/settings.rs

Settings must:

1. Allow configuration of the default revision-session size.
2. Allow configuration of the mastery threshold.
3. Ship with an initial mastery threshold of either 10 or 15 successful revisions while permitting later adjustment after real-world testing.
4. Allow configuration of audio volume or supported audio behavior.
5. Persist changes locally across application and device restarts.
6. Apply new values to subsequent selection, mastery evaluation, and playback without rewriting established learning history.
