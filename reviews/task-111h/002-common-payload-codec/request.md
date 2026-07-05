# Task 111h / 002 — Common Payload Codec Checkpoint

Code commit: `1ba94bdee716ed04a131154afee75cab009d0b5c`

## Summary

This checkpoint adds the common rerank payload codec required by Task 111h and
routes the legacy `0x2A` index-side sidecar through it.

What changed:

- Added a shared `RerankPayloadCodec` surface for compact rerank payloads:
  build/insert encoding, payload length/alignment, query prep, scalar scoring,
  and batch scoring.
- Kept f32 as the `source` baseline with no sidecar payload.
- Kept f16 on index placement, but changed persisted f16 sidecar scoring to
  score directly from packed binary16 bytes instead of unpacking each candidate
  into a temporary `Vec<f32>`.
- Moved RaBitQ-4 sidecar encoding/scoring onto the common codec.
- Added persisted RaBitQ-8 sidecar encoding/scoring through the same codec.
- Added persisted TurboQuant sidecar encoding/scoring through the same codec;
  the rerank payload stores gamma as a fixed f32 prefix followed by TurboQuant
  codec bytes.
- Updated coarse_rerank reloption resolution so `rabitq8` and `turboquant`
  resolve to `rerank_placement = 'index'` under `auto`, and are accepted for
  explicit index placement.

This does **not** claim the final 111h packed group/segment layout is done. The
formats above are now implemented on the legacy direct-TID `0x2A` sidecar
baseline so they can be correctness-tested and then benchmarked before the
packed layout supersedes it.

## Code Changes

- `src/am/ec_ivf/rerank.rs`: introduces the common codec and refactors scan-time
  scoring plus build/insert sidecar encoding to use it.
- `src/am/ec_ivf/options.rs`: accepts RaBitQ-8 and TurboQuant for
  `coarse_rerank` compact index placement, leaving RaBitQ-2 rejected.
- `src/tests/ec_ivf.rs`: extends admin and scan fixtures to cover RaBitQ-8 and
  TurboQuant index placement.
- `src/am/ec_ivf/page.rs`, `src/am/ec_ivf/scan.rs`, `docs/on-disk-format.md`:
  update comments/docs from f16/RaBitQ-4-only sidecar wording to common payload
  codec wording.
- `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md`: marks only the common
  codec and persisted format-encoding checklist items completed.

## Validation

Artifacts are under `reviews/task-111h/002-common-payload-codec/artifacts/`.

- `cargo check --no-default-features --features pg18` passed.
- `cargo test --no-default-features --features pg18 compact_payload_codecs_score_source_and_sidecar_consistently --lib`
  passed: `1 passed; 0 failed`.
- `cargo test --no-default-features --features pg18 coarse_rerank --lib`
  passed: `23 passed; 0 failed`.
- `cargo test --no-default-features --features pg18 index_placement --lib`
  passed: `10 passed; 0 failed`.
- `cargo test --no-default-features --features pg18 index_quant_formats_top_neighbor --lib`
  passed: `1 passed; 0 failed`.

## Review Focus

- Confirm the codec boundary is narrow enough for the future packed
  group/segment layout to reuse without format-specific storage branches.
- Confirm TurboQuant’s sidecar payload shape (`gamma` f32 prefix plus codec
  bytes) is acceptable for the legacy baseline and future packed layout.
- Confirm accepting RaBitQ-8/TurboQuant in `coarse_rerank` options is correct at
  this stage, with the understanding that benchmarking and final promote/abandon
  decisions remain open.
