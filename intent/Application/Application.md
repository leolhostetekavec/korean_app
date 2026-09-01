---
artifact_repo: korean_app
---

# Application

Application is the local-first mobile experience for a native French speaker who is a complete Korean beginner. It prioritizes useful spoken Korean, listening, active recall, understandable pronunciation, and real conversation. V1 does not teach Korean reading or writing and never requires an account or network connection for core learning.

Its artifact is a Rust application crate under crates/app. The crate owns the runtime, interface, learning state, revision behavior, settings, audio coordination, and local persistence. It consumes the public PronunciationEngine.pipeline boundary and versioned course bundles supplied by LanguageContent.

Application is a wrapper around LanguageContent. It accepts the islands, lessons, sentences, ordering, and objectives that LanguageContent provides without defining their structure, cardinality, schema, or pedagogical composition. Sharing an artifact crate does not merge the two intent entities.

The central interaction is:

1. Show the French meaning.
2. Let the learner recall the Korean sentence.
3. Reveal the French-friendly pronunciation on request.
4. Allow replay of authoritative Korean audio.
5. Let the learner self-assess.
6. Repeat material until it is remembered.

V1 remains deliberately focused: spoken communication over literacy, active recall over recognition, sentences over isolated words, authoritative Korean audio over visual approximation, and simple offline reliability over breadth.

V2 may extend the application without replacing its learning system:

1. Hangul may be displayed optionally beside the French-friendly pronunciation and may later become a taught skill.
2. Words associated by LanguageContent with learned sentences may appear in a personal dictionary with Hangul, French-friendly pronunciation, French meaning, individual audio, sentence links, browsing, listening, and separate word practice.
3. Optional notifications may suggest revision when many active sentences have aged, but may never dictate when the learner studies.
4. Learning history may recommend useful session sizes and material needing attention while retaining Application.revision.SessionSelectionContract as the fallback.
5. Speech feedback may be added only when its evaluation is reliable enough to help the learner.
6. Guided or AI-supported conversation may build spontaneous responses from already learned sentences, words, and structures.

These extensions must remain optional where appropriate and must not weaken or replace the central recall loop. They are not V1 commitments.
