---
tests_path: crates/pronunciation-engine/tests
---

# Pipeline

The pipeline is the pronunciation engine's public entry point. It coordinates the complete conversion while keeping internal stages independently testable and preventing the application from reproducing phonological logic.

## LibraryBoundaryContract

- binding: semantic
- artifact_path:
  - crates/pronunciation-engine/Cargo.toml
  - crates/pronunciation-engine/src/lib.rs

The library boundary must:

1. Define the pronunciation engine as a Rust library crate that another workspace crate can import.
2. Disable publication to public package registries.
3. Support consumption through a local path dependency without requiring crates.io.
4. Expose a public end-to-end pronunciation operation while keeping implementation modules private unless another public use is intentionally required.
5. Keep crate, function, parameter, result, and error names agent-chosen until a later contract pins their exact spelling or signature.

## EndToEndContract

- binding: semantic
- artifact_path: crates/pronunciation-engine/src/lib.rs

The public operation must:

1. Accept Korean text containing Hangul for pronunciation.
2. Parse Hangul through PronunciationEngine.parser.
3. Produce Korean surface pronunciation through PronunciationEngine.rules.
4. Produce French-oriented phonetic targets through PronunciationEngine.mapper.
5. Produce French-readable pronunciation through PronunciationEngine.renderer.
6. Return the final French-readable result to the caller.
7. Report invalid or unsupported input through a structured error outcome rather than silently fabricating pronunciation.
8. Preserve the invariant that French adaptation never modifies or determines Korean phonology.
