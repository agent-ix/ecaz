# Task 87 Packet 021: SPIRE Leaf LUT32 Batching

## Scope

This packet covers the follow-up to packet 020 where SPIRE real10k exercised `CandidateBatch` but did not reach the 32-wide LUT kernel because each V2 leaf segment was scored separately.

Code commit under review:

- `56299f37fdce4300dfba11ab5b63f21284adb6bd` - `Batch SPIRE leaf column scoring for Task 87`

Implementation summary:

- Added `SpirePreparedAssignmentScorer::score_candidate_batch_ip`, preserving the common `CandidateBatch` scorer/counter/LUT32 path.
- Changed SPIRE V2 leaf candidate scanning to gather adjacent leaf column segments into one leaf-level `CandidateBatch` when selected-row pruning is not active and RaBitQ bounded cutoff is not required.
- Preserved the existing per-column path for selected row ranges and bounded RaBitQ cutoff scans.
- Restored the old payload stride validation behavior before entering the no-QJL LUT scorer so malformed TurboQuant payloads still fail with the expected SPIRE stride mismatch error instead of panicking inside product quantizer helpers.

## Validation

Focused tests:

- `cargo test --lib am::ec_spire::quantizer --no-default-features --features pg18`
  - `15 passed; 0 failed`
- `cargo test --lib am::ec_spire::scan --no-default-features --features pg18`
  - `99 passed; 0 failed`
- `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
  - `4 passed; 0 failed`

PG18 install:

- Installed `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Installed backend SHA256: `b7cdee8d972cd7f45725a8875116a47f89647700d5920d1fe0e42e005bf158c2`
- Verified task 87 counter SQL functions are registered in `postgres`.

Suite:

- Config: `reviews/task-87/021-spire-leaf-lut32-batching/phase7-real10k-counter-suite.json`
- Status: `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

Key results:

- IVF recall off/on remained `1.0000`.
- IVF candidate-batch-on latency: `p50=16.7 ms`, `p95=18.0 ms`, `p99=21.1 ms`.
- IVF on counters: `surface=ivf flushes=8000 candidates=2000000 lut32_flushes=7800 lut32_candidates=1996800`.
- SPIRE candidate-batch-off pipeline: `p50=17.686 ms`, `p95=20.579 ms`, `p99=22.502 ms`, `recall@k=1.0000`.
- SPIRE candidate-batch-on pipeline: `p50=15.413 ms`, `p95=17.951 ms`, `p99=22.600 ms`, `recall@k=1.0000`.
- SPIRE on counters now hit the LUT32 path: `surface=spire flushes=4800 candidates=1551640 lut32_flushes=4800 lut32_candidates=1551640`.
- HNSW real10k still reports zero Task 87 candidate-batch counters for this profile: `surface=hnsw flushes=0 candidates=0 lut32_flushes=0 lut32_candidates=0`.

Artifacts:

- Manifest: `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/manifest.md`
- Suite run manifest: `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/real10k-run-manifest.json`
- Suite results: `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/real10k-results.jsonl`
- Suite status: `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/real10k-status.log`
- Raw run logs: `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/run/`
- Focused test logs:
  - `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/test-ec-spire-quantizer.log`
  - `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/test-ec-spire-scan.log`
  - `reviews/task-87/021-spire-leaf-lut32-batching/artifacts/test-common-candidate-batch.log`

## Notes

Packet 021 closes the packet 020 SPIRE gap: the real10k SPIRE TurboQuant path now reaches the 32-wide LUT kernel through the shared candidate-batch scorer.

This does not change the Task 91/QuantCodec migration boundary for DiskANN. It also does not claim HNSW LUT32 coverage from this real10k profile, because the instrumented HNSW run still does not exercise the Task 87 common candidate-batch scorer.
