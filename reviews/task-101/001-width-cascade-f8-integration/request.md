# Task 101 Packet 001: Shared Width-Cascade Driver

This packet is the first Task 101 implementation checkpoint, coordinated with Task 94 packet 026.

Code checkpoint: `11f8fc38113c08614c8ddca2073e54adcb018d81` (`Unify batch width cascade for quant kernels`)

## What Changed

- Split `src/am/common/candidate_batch.rs` into:
  - `src/am/common/candidate_batch/mod.rs`
  - `src/am/common/candidate_batch/counters.rs`
  - `src/am/common/candidate_batch/drivers.rs`
- Added a shared width-cascade driver:
  - prevalidated inputs stay outside the mutation path;
  - 32-wide blocks are handled centrally;
  - family-specific remainder closures handle octets, partial kernels, or scalar fallback;
  - scalar block fallback attribution is explicit per family.
- Migrated the current batch families onto the shared driver:
  - TurboQuant no-QJL lut32
  - TurboQuant QJL
  - TurboQuant tiled LUT
  - TurboQuant int8 approximate
  - RaBitQ bits=1
  - grouped-PQ
  - binary hamming
- Added `QuantCodecKind::TurboQuantTiledLut` and `QuantCodecKind::TurboQuantInt8`; `QuantCodecKind::ALL` is now 7-wide.
- Kept Task 87 compatibility behavior keyed only to `QuantCodecKind::TurboQuant`.
- Backported partial-width dispatch for lut32 and grouped-PQ by padding sub-32 tails into the block scorer and copying live lanes back.
- Backported no-QJL prevalidation so candidate shape/meta validation completes before score output mutation.

## Local Evidence

Artifacts: `reviews/task-101/001-width-cascade-f8-integration/artifacts/`

| Command | Result |
| --- | --- |
| `cargo test --lib candidate_batch` | 18 passed, 0 failed |
| `cargo test --lib grouped_pq` | 35 passed, 0 failed |
| `cargo test --lib qjl32` | 11 passed, 0 failed |
| `cargo test --lib hamming32` | 3 passed, 0 failed |
| `cargo test --lib int8_approx32` | 2 passed, 0 failed |
| `cargo test --lib tiled_lut32` | 1 passed, 0 failed |

Task 94 packet 026 carries the local `ecaz bench suite` matrix evidence for grouped-PQ IVF/DiskANN counter rows under the shared driver:

- suite status: `completed=14 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- direct rows include `surface=ivf quant=grouped_pq isa=avx2 scalar_candidates=0`
- direct rows include `surface=diskann quant=grouped_pq isa=avx2 scalar_candidates=0`

## Notes For Review

- This packet does not claim Task 101 completion. It lands the shared driver, counter split, file split, no-QJL prevalidation, and lut32/grouped-PQ partial backport in one checkpoint.
- The driver intentionally preserves legacy counter semantics where older block32 scalar fallbacks were already counted as kernel rows with `isa=scalar`, while newer partial-capable scalar paths remain scalar rows.
- Graviton 4 evidence remains deferred until AWS testing is approved.

Please review Task 101 packet 001 together with Task 94 packet 026.
