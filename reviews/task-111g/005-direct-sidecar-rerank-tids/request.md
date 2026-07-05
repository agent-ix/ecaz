# Task 111g: Direct Sidecar Rerank TIDs

## Summary

This packet fixes the index-side rerank sidecar lookup path that made the f16
sidecar look pathologically slow at larger corpus sizes. The old ADR-079
directory path still had to materialize/scan a size-proportional directory each
query. That made f16 at 100k measure around `150 ms` even though the compact
payload itself is smaller than f32.

The new hot path persists the physical `0x2A` sidecar block TID on each posting.
Rerank now reads only the sidecar blocks for the survivor candidates. The
directory loader and full-chain loader remain compatibility fallbacks when a
candidate does not have a direct sidecar TID.

Implementation commit: `a7cdb86fe021fa11db0ea00ac07c47c8896d7f1a`.

## Code Changes

- `src/am/ec_ivf/build.rs`: builds the compact rerank sidecar before postings,
  returns a `heap_tid -> sidecar_block_tid` map, and writes direct `rerank_tid`
  values into row and dense postings.
- `src/am/ec_ivf/insert.rs`: appends the insert sidecar block before posting
  construction and stores that TID on the inserted posting; metadata head is
  updated in the normal metadata stats update.
- `src/am/ec_ivf/scan.rs`: carries `rerank_tid` through candidate collection,
  prefers direct sidecar block reads, and falls back to directory/full-chain
  lookup for candidates without direct pointers. Multi-heaptid postings suppress
  the direct pointer to avoid ambiguous lookup.
- `src/am/ec_ivf/page.rs`: exposes dense posting `rerank_tid` accessors.
- `docs/on-disk-format.md`: documents direct posting-carried sidecar TIDs as
  the hot path and the ADR-079 directory as fallback.

## Validation

Artifacts are under
`reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/`.

- `cargo test --no-default-features --features pg18 posting_scratch_soa`
  passed: `5 passed; 0 failed`.
- `cargo check --no-default-features --features pg18` passed.
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
  installed a release build.
- The sidecar-index placement suite completed:
  `completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

## Benchmark Result

The exact suite/config used is copied to
`artifacts/sidecar-index-direct-tids/suite-config.json`; normalized results are
in `artifacts/sidecar-index-direct-tids/results.jsonl`.

Key p50 latency rows:

| cell | nprobe 8 | nprobe 64 | nprobe 200 |
| --- | ---: | ---: | ---: |
| f16 index 50k, old ADR-079 packet | 78.3 ms | 80.6 ms | 91.2 ms |
| f16 index 50k, direct TID packet | 3.79 ms | 4.62 ms | 8.95 ms |
| f16 index 100k, old ADR-079 packet | 146.8 ms | 150.2 ms | 159.2 ms |
| f16 index 100k, direct TID packet | 2.99 ms | 6.02 ms | 13.0 ms |
| rabitq4 index 100k, old ADR-079 packet | 7.67 ms | 9.60 ms | 16.0 ms |
| rabitq4 index 100k, direct TID packet | 2.79 ms | 5.72 ms | 11.9 ms |

Conclusion: the `150 ms` f16 number was real for the previous code, but it was
not evidence that f16 scoring/storage is inherently slow. The size-shaped
latency was caused by the sidecar lookup path doing per-query work that scaled
with sidecar/page count.

## Review Focus

- Confirm the direct sidecar TID semantics are correct for row postings, dense
  postings, and inserted postings.
- Check the fallback behavior for old/ambiguous postings. Multi-heaptid
  postings intentionally mark candidates as `INVALID` for direct sidecar lookup
  and use the directory/full-chain path.
- Check whether any additional insert/vacuum pg_test coverage is required
  before accepting this path; the suite exercises fresh builds and scans, not a
  live insert followed by rerank.
