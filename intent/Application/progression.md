---
tests_path: crates/app/tests
---

# Learning Progression

This component tracks movement through the course supplied by LanguageContent without defining that course.

## SequentialProgressContract

- binding: semantic
- artifact_path: crates/app/src/progression.rs

Progression must:

1. Respect the island and lesson order supplied by LanguageContent.
2. Make the first supplied lesson available when no prior progress exists.
3. Unlock the next supplied lesson only after the current lesson satisfies Application.lesson.LessonSessionContract completion.
4. Mark an island complete after its final supplied lesson is complete.
5. Unlock the next supplied island after the current island completes.
6. Handle course size and grouping entirely from supplied content rather than application constants.

## ProgressOverviewContract

- binding: semantic
- artifact_path: crates/app/src/progression.rs

The progress model must expose:

1. Completed lessons and islands.
2. The current lesson.
3. Sentence learning states.
4. Practice-pool membership.
5. Active revision membership.
6. Mastered sentence membership.
7. Revision counts and last-revised timestamps.
8. Any unfinished lesson or revision session.
