# Task 125-129 Closeout Audit

- task bucket: `reviews/task-125/002-closeout-audit`
- audited branch/head: `task-125-tq-scorer-optimization` at `371db1bdc`
- audited source task definitions: `origin/task-124-ivf-tq-stage2:plan/tasks/125-tq-scorer-lut-memory-traffic.md` through `129-tq-payload-conversion-cleanup.md`
- timestamp: `2026-07-01T17:51:00Z`
- prior implementation packet: `reviews/task-125/001-tq-scorer-optimization/`

## Summary

The current branch has a strong accepted scorer optimization:

- compact no-QJL 4-bit LUT: `f32` table -> `i16 + scale`;
- `BLOCK_WIDTH = 64`;
- NEON sparse suffix-bound pruning;
- TurboQuant no-QJL score-buffer reuse / direct negated output;
- no-QJL gamma side-input avoidance in the hot batch scorer.

The committed suite evidence shows unchanged recall/storage and improved
latency/kernel time versus the prior sparse-bound candidate:

- 10k: latency mean `1.22 ms -> 0.88 ms`, NEON kernel `46.240597 ms -> 23.924550 ms`;
- 50k: latency mean `2.58 ms -> 1.88 ms`, NEON kernel `76.955763 ms -> 41.573022 ms`;
- 100k: latency mean `3.86 ms -> 2.73 ms`, NEON kernel `74.883089 ms -> 38.896097 ms`.

This is not enough to prove literal closeout of every explicit requirement in
Tasks 125-129.

## Requirement Matrix

| Task | Requirement | Current evidence | Status |
| --- | --- | --- | --- |
| 125 | Cache-block / tile over dimensions by restructuring from "per candidate-block: walk all dims" to "per dim-tile: walk all candidate-blocks". | `src/am/common/candidate_batch/drivers.rs` still implements `score_width_cascade` as a block loop. `src/am/common/candidate_batch/mod.rs` still calls `score_lut_no_qjl_4bit_block32` per block. `src/quant/lut32/neon.rs` chunks dimensions inside one candidate block, but does not load a dim tile once and walk all candidate blocks. | Missing / not proven |
| 125 | Shrink the prepared LUT to f16 or int16 and prove recall safety. | `src/quant/prod.rs` defines `PreparedLutNoQjl4BitQuery { lut: Vec<i16>, lut_scale: f32, suffix_max }`, `build_prepared_query_lut_i16`, and `build_lut_suffix_max_i16`. `candidate-int16-lut` suite has unchanged recall at 10k/50k/100k. | Complete |
| 126 | Raise/parameterize `BLOCK_WIDTH` and sweep against 32/64/128, preserving tails. | `src/quant/lut32/mod.rs` has `BLOCK_WIDTH: usize = 64`. `cargo test -p ecaz --lib lut32_ -- --test-threads=1` passed, covering tails and scalar/backend parity. No committed 32/64/128 curve is present. | Partial |
| 127 | Dimension-progressive SIMD pruning with exact recall and pruned fraction. | `src/quant/lut32/neon.rs` has `score_octets_neon_with_min_bound_impl`, `BOUND_CHECK_DIM_STRIDE = 512`, live-lane pruning, and final keep flags. `candidate-t127-sparse` and `candidate-int16-lut` preserve recall. No committed pruned-fraction metric is present. | Partial |
| 128 | Reuse scratch buffers across flushes and fold the estimates negate pass for the TurboQuant no-QJL scorer path. | `src/am/ec_ivf/scan.rs` routes TurboQuant no-QJL scratch SoA scoring through scan-owned `scratch.scores` / `scratch.kept`. `src/am/ec_ivf/rerank.rs` uses `score_turboquant_batch_from_payloads_negated_into` for TurboQuant payload slabs. | Complete for covered TQ no-QJL path |
| 129 | Drop unused no-QJL gamma parse and collapse repeated pointer-vector rebuilding. | `src/am/common/candidate_batch/mod.rs` validates no-QJL metadata with `CandidateMeta::None` / `Gamma(0.0)`, scores no-QJL payloads through `mse_code_bytes_no_qjl_4bit`, and passes no gamma side input. `cargo test -p ecaz --lib turboquant_no_qjl -- --test-threads=1` passed. | Complete for covered TQ no-QJL path |

## Validation Already Present

From `reviews/task-125/001-tq-scorer-optimization/request.md`:

- `cargo fmt --check`
- `cargo check -p ecaz --lib`
- `cargo test -p ecaz --lib lut32_ -- --test-threads=1`
- `cargo test -p ecaz --lib explicit_lut -- --test-threads=1`
- `cargo test -p ecaz --lib turboquant_no_qjl -- --test-threads=1`
- `cargo test -p ecaz --lib turboquant_lut_bounded_batch_keeps_and_prunes -- --test-threads=1`
- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `target/release/ecaz bench suite run ...` at 10k/50k/100k for recall, latency, and storage.

## Closeout Decision

Do not mark the full 125-129 objective complete on the current evidence.

Recommended next implementation packet:

1. Implement true cross-block dimension cache-blocking for the no-QJL 4-bit
   scorer, or explicitly amend Task 125 to accept the int16 compact-LUT result
   as the cache-traffic solution in place of cross-block tiling.
2. Produce the Task 126 32/64/128 width curve with `ns/candidate` evidence.
3. Add/report a Task 127 pruned-fraction metric for the bounded NEON scorer.

After those are present, rerun the same 10k/50k/100k suite and update
`reviews/task-125/001-tq-scorer-optimization/artifacts/manifest.md` or a new
closeout packet with the final evidence.

## Superseded Closeout Note: 2026-07-01

This audit was accurate for audited head `371db1bdc`, but the missing closeout
items were subsequently addressed:

- Task 125: code commit `96782e209010a70538e94c63dd46e8b2dd54cec2` added
  `score_lut_no_qjl_4bit_batch_tiled` and the NEON `score_batch_tiled_neon`
  path, which walks each dimension chunk across every candidate octet before
  moving to the next chunk.
- Task 126: `task126-width-profile.log` reports the 32/64/128 width curve at
  `12662.2`, `11578.1`, and `11084.1 ns/candidate`.
- Task 127: heap-rerank TurboQuant IVF logs now report exact prune-on/prune-off
  recall parity plus bounded prune fractions at 10k/50k/100k:
  - 10k: recall `1.0000 -> 1.0000`; `98.3%` bounded prune fraction.
  - 50k: recall `0.9641 -> 0.9641`; `97.7%` bounded prune fraction.
  - 100k: recall `0.9268 -> 0.9268`; `97.8%` bounded prune fraction.

The superseding evidence lives in
`reviews/task-125/001-tq-scorer-optimization/artifacts/manifest.md`.
