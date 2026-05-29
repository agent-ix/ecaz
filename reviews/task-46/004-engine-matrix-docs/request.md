# Task 46/004: engine matrix docs (closes §Exit Criteria #5)

## Scope

Closes Task 46 §Exit Criteria gate 5:

> 5. `docs/hardening.md` documents the engine matrix and corpus management.

Validation head: `ddb51741e`. Single-commit docs slice (also closes
Task 48 §Exit #4 via the companion file `docs/build-matrix.md` —
that packet is `reviews/task-48/002-build-matrix-docs/`).

## What changed

- `docs/hardening.md` — adds an "Engine Matrix" subsection under
  `## Fuzzing`. Documents the five fuzz lanes (libFuzzer,
  Honggfuzz, AFL+, ECAZ-grammar SQLsmith, cross-pollinate), the
  decoder-vs-structured-input target shape taxonomy from Task 46
  §Why, the committed-corpus workflow established by packet 003,
  and the criteria for adding a new structured target.

No production code change. Documentation-only.

## Reviewer focus

- The matrix table enumerates the five lanes named in Task 46
  §Approach with cadence + strengths + notes columns. Cross-
  pollinate appears as a row even though that lane will be wired up
  by a future Honggfuzz/AFL+ integration slice, so the
  documentation does not lag the spec.
- Target shape taxonomy explicitly preserves the decoder targets'
  raw-byte input (per Task 46 §Why: "For decoders this is correct —
  the input *is* bytes"), avoiding the failure mode of converting
  decoders to Arbitrary and obscuring what they test.
- Corpus management subsection cross-references the actual workflow
  established by packet 003 (`make fuzz-corpus-minimize`,
  `.gitignore` policy) so the docs and the code agree.
- "When to add a new structured target" lists three concrete
  triggers + four requirements that match the slice 001/002 shape.

## Out of scope (other Task 46 gates)

- §Exit #1 (every structured-input target uses Arbitrary): partial
  — 2 of N done. Doc lists the taxonomy; per-target conversion is
  per-slice.
- §Exit #2 (`make sqlsmith-ecaz` nightly): doc names the lane,
  implementation is a follow-up slice.
- §Exit #3 (Honggfuzz + AFL+ weekly): doc names the lanes,
  implementation is a follow-up slice.
- §Exit #4: closed by packet 003.

This packet closes §Exit #5 only. Task 46 now ~35% complete (2 of
5 gates closed; one partial).
