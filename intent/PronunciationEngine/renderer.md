# French-Readable Renderer

The renderer converts French-oriented phonetic targets into text that a native French reader can pronounce approximately. It is a presentation convention, not a Korean phonology component.

## FrenchReadableOutputContract

- binding: semantic
- artifact_path: src/french.rs

The renderer must:

1. Accept only the French-oriented target sequence from the mapper.
2. Keep spelling decisions separate from Korean parsing and Korean phonological transformations.
3. Use readable French conventions where they improve pronunciation, including conventions such as ou, eu, é, euh, ya, yo, you, wa, wé, ng, and ch where applicable.
4. Preserve enough visible distinction between Korean plain, aspirated, and fortis targets to avoid collapsing all three into one French spelling.
5. Avoid presenting the result as Revised Romanization, McCune-Reischauer, English phonetics, or visually similar Latin spelling.
6. Return deterministic output for a given French-target sequence under the active rendering convention.
7. Keep the rendering convention replaceable independently from the Korean engine, allowing future English, Spanish, Slovenian, or other target-language renderers.
8. Support end-to-end inspection of the progression from Hangul through parsed structure, Korean surface pronunciation, French target, and final readable output.

