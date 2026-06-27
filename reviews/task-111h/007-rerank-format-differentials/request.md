# Task 111h / 007 - Rerank Format Differential Test Checkpoint

## Scope

Code commit: `bd72a9cc18b39c2d5176a4673d4d1bc492f1eeae`

This checkpoint closes the Task 111h differential-test checklist item for the
implemented rerank formats:

- Adds a compact-format differential test covering `rabitq4`, `rabitq8`, and
  `turboquant`.
- The test compares:
  - scalar source-diagnostic scoring,
  - source batch scoring,
  - persisted sidecar scalar scoring,
  - persisted sidecar batch scoring.
- Keeps the existing f32 source reference and f16 encode/decode/direct-payload
  tests as coverage for the non-batched formats.
- Marks the task checklist item for encode/decode and scalar-vs-batch
  differential coverage complete.

## Non-Claims

This packet is not benchmark evidence. It does not make any latency, recall, or
storage decision for the formats; it only verifies that the implemented scoring
surfaces agree on small deterministic vectors.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo-test-rerank-batch-differential.log`: `cargo test --no-default-features --features pg18 compact_payload_codecs_batch_paths_match_scalar_scores --lib`
  passed with 1 test.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.

## Review Focus

- Verify the test covers the required compact rerank formats (`rabitq4`,
  `rabitq8`, `turboquant`) across source and persisted-payload scoring paths.
- Verify the tolerance is appropriate for approximate quantized scoring and is
  not hiding unrelated drift.
- Verify the task checklist update is justified by this test plus the existing
  f32/f16 rerank tests.
