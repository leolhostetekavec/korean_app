---
tests_path: crates/app/tests
---

# Learner Interface

The learner interface is French, simple enough for a complete beginner, and centered on one repeatable recall interaction.

## V1 UI implementation direction

V1 uses Slint as its declarative UI framework. The interface remains implemented in
Rust within the Application crate, while Android-specific services are accessed through
thin platform adapters. The semantic interface contracts below remain authoritative
over the framework implementation.

## RecallInteractionContract

- binding: semantic
- artifact_path: crates/app/src/ui.rs

Every lesson, practice, and revision recall card must:

1. Initially display only the French prompt.
2. Wait for the learner to attempt the Korean answer from memory.
3. Offer the exact action lit:Voir la réponse.
4. Reveal the French-friendly pronunciation only after that action.
5. Enable the exact audio action lit:Écouter after the answer is revealed.
6. Offer the exact outcomes lit:Je connais and lit:Je ne connais pas after the answer is revealed.
7. Keep Hangul hidden in V1.
8. Avoid passive recognition hints that bypass recall before the answer is revealed.

## ScreenSetContract

- binding: semantic
- artifact_path: crates/app/src/ui.rs

The V1 interface must provide:

1. A home screen.
2. An island overview supplied from LanguageContent.
3. A lesson list for the selected island.
4. A lesson introduction and learning screen.
5. A practice screen.
6. A revision-session setup screen.
7. A revision screen.
8. A progress overview.
9. Basic settings.
10. French interface copy and navigation that do not require prior Korean knowledge.
