# Task 29c DiskANN Build Phase Profile

## Request

Review the first Task 29c build-performance slice: structured ambuild phase
timing plus one measured optimization attempt.

Current branch head: `11393c34`

Code commits in this slice:

- `1728b943` adds structured DiskANN ambuild phase timing.
- `05f38e73` surfaces the single timing line to client-visible logs so
  `ecaz-cli dev sql --log-output` can capture it.
- `d2e0e9fc` / `36f0c3d5` tried a heap frontier for in-memory Vamana build
  search.
- `11393c34` reverts that heap experiment after measurement showed a
  regression.

## Result

Local PG18, isolated real-10k 1536-d corpus, index-only `CREATE INDEX` with
`ec_diskann` reloptions `graph_degree=32`, `build_list_size=100`, `alpha=1.2`:

| build | total_ms | build_persist_ms | payload_ms | training_ms | write_pages_ms |
|---|---:|---:|---:|---:|---:|
| baseline with phase timing | 490,850 | 471,757 | 10,187 | 4,429 | 47 |
| heap-frontier build search | 551,334 | 524,053 | 11,652 | 5,214 | 43 |

The phase split is decisive: page persistence/WAL is not the 10k build blocker
in this run. `build_persist_ms` is about 96% of total build time; data page
write time is effectively zero at this scale.

The heap-frontier experiment regressed total build time by about 12%, so it was
reverted. The next useful optimization is not frontier candidate selection. It
is finer instrumentation inside `build_vamana_graph`:

- greedy-search elapsed time and visited-count distribution per pass;
- robust-prune elapsed time and candidate-pool distribution per pass;
- build-distance call counts and time, especially source-vector dot products.

After those counters, attack whichever of distance evaluation or robust-prune
dominates. If distance calls dominate, the likely direction is caching/reusing
candidate distances or changing the build distance representation. If prune
dominates, look at candidate-pool reduction and skip/fast-path cases before
considering algorithmic changes.

## Validation

Timing code:

- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes`
- `git diff --check`

Heap experiment and revert:

- `cargo test --lib am::ec_diskann::vamana -- --nocapture`
- `cargo test --lib am::ec_diskann::scan -- --nocapture`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

After pg_test runs, the normal PG18 extension build was reinstalled and the
local PG18 server was restarted. After the final revert, normal PG18 was
reinstalled and restarted again so the local server matches branch HEAD.

## Recommendation

Do not land Task 29 yet on build-performance grounds. Task 29b is ready for
outside review, but Task 29c still needs one deeper profile pass inside the
in-memory Vamana core and reference comparison against `ec_hnsw` and
pgvectorscale before the landing decision is defensible.

Raw logs are under `artifacts/`; see `artifacts/manifest.md`.
