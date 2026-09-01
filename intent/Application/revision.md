---
tests_path: crates/app/tests
---

# Long-Term Revision

This component selects learned sentences for learner-initiated revision and moves stable sentences out of normal selection.

## RevisionRecordContract

- binding: semantic
- artifact_path: crates/app/src/revision.rs

Each learned sentence revision record must:

1. Identify the sentence with lit:sentence_id.
2. Store the timestamp lit:last_revised.
3. Store the successful revision total lit:revision_count.
4. Store lit:status as either lit:ACTIVE or lit:MASTERED.
5. Use no automatically calculated next-revision or due date in V1.
6. Retain the record and its history when its status becomes lit:MASTERED.

## SessionSelectionContract

- binding: semantic
- artifact_path: crates/app/src/revision.rs

When the learner requests a revision session, selection must:

1. Read the requested sentence count without imposing a daily limit.
2. Select lit:ACTIVE records whose lit:revision_count is 0 first, ordered by lit:last_revised from oldest to newest.
3. Fill any remaining places from the other lit:ACTIVE records, ordered by lit:last_revised from oldest to newest.
4. Use every available active sentence when fewer exist than requested.
5. Exclude lit:MASTERED records from normal automatic selection.
6. Freeze the selected sentence identities as one session before presentation begins.
7. Allow the learner to create multiple sessions on the same day.

## RevisionCycleContract

- binding: semantic
- artifact_path: crates/app/src/revision.rs

An active revision session must:

1. Use Application.interface.RecallInteractionContract for every selected sentence.
2. On lit:Je connais, remove the sentence from the current session, set lit:last_revised to the current timestamp, and increase lit:revision_count by 1.
3. On lit:Je ne connais pas, leave lit:last_revised and lit:revision_count unchanged and retain the sentence in the session.
4. Present a retained failed sentence again after the other remaining sentences.
5. Complete only after every originally selected sentence has received lit:Je connais at least once in that session.
6. Restore the exact unfinished session after navigation away, application closure, or restart.

## MasteryContract

- binding: semantic
- artifact_path: crates/app/src/revision.rs

Mastery evaluation must:

1. Read the current configurable mastery threshold.
2. Evaluate the updated lit:revision_count after every successful revision.
3. Change lit:status to lit:MASTERED when the successful count reaches or exceeds the threshold.
4. Preserve the sentence record, revision count, timestamp, and learning history after mastery.
5. Keep mastered sentences available to progress and future features while excluding them from normal revision selection.
