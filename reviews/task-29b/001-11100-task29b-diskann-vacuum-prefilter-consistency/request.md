# Task 29b DiskANN Vacuum Prefilter Consistency

## Request

Review the Task 29b follow-up that makes DiskANN scan and vacuum repair use the
same prefilter selection path.

Measured commit: `95fef9acca515c1dee61d6195085c62f5362779f`

## Change

`src/am/ec_diskann/routine.rs` now has a shared `PreparedPrefilter` helper used
by both `amrescan` and vacuum-repair candidate planning:

- `auto` uses the persisted binary sidecar when the index has it;
- `binary_sidecar` requires the sidecar and errors if it is absent;
- `grouped_pq` forces the legacy grouped-PQ fallback.

The change keeps grouped-PQ as the explicit rollback path, updates the
`ec_diskann.prefilter_kind` GUC doc string to that production intent, adds a
PG18 pg_test for the prefilter override, and simplifies the scan frontier pop
helper flagged in the merge-readiness review.

## Result

Local PG18 isolated real-10k vacuum fixture:

| phase | live rows | list_size | recall@10 | NDCG | mean q-time |
|---|---:|---:|---:|---:|---:|
| pre-vacuum | 10,000 | 200 | 0.9970 | 0.9999 | 52.52 ms |
| post-delete/vacuum | 9,500 | 200 | 0.9975 | 0.9999 | 52.33 ms |

Procedure:

1. Loaded isolated prefix `task29b_vacuum_real10k` with `ec_diskann` and
   reloptions `graph_degree=32`, `build_list_size=100`, `alpha=1.2`.
2. Confirmed pre-vacuum recall against a fresh 10k-row truth cache.
3. Deleted exactly 500 rows via `id % 20 = 0`.
4. Ran `VACUUM (ANALYZE)` on the isolated corpus table.
5. Recomputed truth for the 9,500 live rows and remeasured recall.

The post-vacuum result stays above the Task 29 scan-path floor of `0.99`, so
the sidecar vacuum-repair path is recall-neutral on this fixture.

## Codegen Check

`cargo asm` is not installed locally, so I used `cargo rustc --emit=asm` with
`-C target-cpu=native -C link-dead-code` and captured the generated function
body. The `hamming_xor_popcount` body uses AVX2 vector operations for the bulk
loop (`vpxor`, `vpshufb`, `vpsadbw`, `vpaddd`) and `popcntq` for the scalar
tail/closure. No hot-path rewrite is needed for Task 29b.

## Validation

- `cargo test --lib am::ec_diskann::scan -- --nocapture`
- `cargo test --lib am::ec_diskann::routine -- --nocapture`
- `cargo test --lib am::ec_diskann::vacuum -- --nocapture`
- `cargo pgrx test pg18 test_ec_diskann_vacuum_refills_broken_neighbor_slot`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

After the pg_test run, the normal PG18 extension build was reinstalled and the
local PG18 server was restarted.

## Recommendation

Task 29b is ready for outside review. The remaining Task 29 landing blocker is
Task 29c build-performance profiling: the latest isolated build still shows the
same shape as earlier packets (`494.39s` index build, `505.57s` total load) and
needs the structured timing/profile pass before a final landing call.

Raw logs are under `artifacts/`; see `artifacts/manifest.md`.
