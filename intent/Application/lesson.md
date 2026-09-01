---
tests_path: crates/app/tests
---

# Lesson Learning

This component introduces a supplied lesson in order and retains unknown sentences in practice until recalled.

## LessonSessionContract

- binding: semantic
- artifact_path: crates/app/src/lesson.rs

A lesson session must:

1. Introduce every sentence supplied for the lesson exactly once in its supplied order before introduction ends.
2. Use Application.interface.RecallInteractionContract for each introduction.
3. On lit:Je connais, add the sentence to active revision with lit:revision_count equal to 1 and lit:last_revised equal to the current timestamp.
4. On lit:Je ne connais pas, add the sentence to that lesson's practice pool without adding a successful revision.
5. Continue into practice after every supplied sentence has been introduced when the practice pool is not empty.
6. Complete only after every supplied sentence has been introduced and the practice pool is empty.
7. Preserve the exact unfinished position, outcomes, and practice membership when the learner leaves.

## PracticeCycleContract

- binding: semantic
- artifact_path: crates/app/src/lesson.rs

Practice behavior must:

1. Use Application.interface.RecallInteractionContract.
2. Present every sentence currently in the lesson's practice pool.
3. On lit:Je connais, remove the sentence from practice and add it to active revision with lit:revision_count equal to 0 and lit:last_revised equal to the current timestamp.
4. On lit:Je ne connais pas, retain the sentence in practice and present it again after the other remaining sentences in the current cycle.
5. Continue until the practice pool is empty.
6. Preserve the exact unfinished cycle when the learner leaves or the application closes.
