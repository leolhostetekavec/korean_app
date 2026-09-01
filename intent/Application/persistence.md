---
tests_path: crates/app/tests
---

# Local Persistence

This component keeps the complete V1 experience usable offline and prevents learning progress from being lost.

## OfflineOperationContract

- binding: semantic
- artifact_path: crates/app/src/persistence.rs

After installation and content bundling or download, the application must operate without an internet connection by:

1. Browsing all supplied course content locally.
2. Playing referenced Korean audio locally.
3. Running lesson introduction and practice locally.
4. Creating and completing revision sessions locally.
5. Reading settings and progress locally.
6. Saving all resulting state locally.

## RecoveryContract

- binding: semantic
- artifact_path: crates/app/src/persistence.rs

Persistence must:

1. Save completed lessons and islands, the current lesson, sentence states, practice pools, active and mastered revision records, counts, and timestamps.
2. Save unfinished lesson position and unfinished revision-session membership.
3. Restore saved state after application closure and device restart.
4. Preserve saved state across application updates that retain application data.
5. Require neither a user account nor cloud synchronization.
6. Avoid resetting valid progress when supplied content is reopened.
