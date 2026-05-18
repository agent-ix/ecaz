# Review Request: C1 ADR-030 V2 PqFastScan Runtime Fixture Contract Alignment

Current head: `a4ccba9`

This packet covers local uncommitted work on top of that head.

## Context

After packets `404`, `408`, `410`, and `411`, the branch's actual
`pq_fastscan` runtime behavior was stronger and more specific than several of
the pg tests still assumed:

1. source-backed default rerank now uses raw heap `source`, not the quantized
   `embedding`
2. binary traversal tests need a genuinely binary-capable `pq_fastscan` layout,
   not a small 16-dimension fixture with a hard-coded `1` word expectation
3. larger live-window settings do not have to reorder output on every
   deterministic query, they only need to preserve the correct emitted set and
   comparison bookkeeping
4. some round-trip/vacuum tests were still hard-coding row ids that no longer
   self-rank first on the current fixtures

The result was not a runtime defect. It was a test contract lagging the
current source-backed implementation.

## Problem

Before this slice, multiple pg tests were asserting stale expectations:

1. exact comparison score tests used `embedding <#> query` where the runtime was
   intentionally using the raw `source` column
2. binary runtime-settings/profile tests were not built on a real binary lane
3. live-window tests required movement even in cases where the wider window
   legitimately kept the same order
4. insert/vacuum round-trip tests assumed specific row ids rather than choosing
   rows that the current fixture actually ranks first for themselves

That produced false negatives once the code path was repaired.

## Planned Slice

Realign the pg test contracts with the current runtime:

1. compute expected exact scores from the raw `source` vectors on
   source-backed lanes
2. add a dedicated binary-capable `pq_fastscan` runtime fixture
3. compare floats with a small tolerance at the operator-facing score surface
4. only require live-window reordering where the chosen query actually moves
5. choose delete/insert candidates by observed self-rank instead of fixed ids

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`

Concrete changes:

1. added `assert_f32_close(...)` for operator-facing score assertions
2. changed the source-backed exact-score expectations to derive their expected
   values from the raw source vectors instead of SQL over the quantized
   `embedding`
3. added a binary-capable runtime fixture in `src/lib.rs`:
   - `create_pq_fastscan_binary_runtime_fixture(...)`
   - `pq_fastscan_binary_runtime_query()`
   - `PQ_FASTSCAN_BINARY_RUNTIME_WORD_COUNT`
4. updated the binary runtime settings/profile tests to use that fixture and the
   computed word-count expectation instead of `Some(1)`
5. tightened the exact-traversal and heap-rerank assertions so they compare:
   - emitted order-by score
   - comparison sidecar score
   - exact expected source score
6. changed `assert_pq_fastscan_runtime_live_window_matches_windowed_simulation`
   so callers can distinguish:
   - cases that must prove movement
   - cases that only need to prove the emitted set and score bookkeeping still
     match the simulation
7. updated the round-trip and vacuum tests to choose deleted/inserted rows by
   observed self-rank instead of assuming `id = 1` or `id = 17`
8. reordered the small source-backed fixture used by the rerank-parity coverage
   so its expected ordering matches the current source-backed runtime behavior
9. tightened `grouped_binary_traversal_score_enabled(...)` in `src/am/scan.rs`
   so binary traversal only activates for actual
   `GraphStorageDescriptor::PqFastScan(_)` layouts

## Validation

Passed:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Still failing in this environment:

- `bash scripts/run_pgrx_pg17_test.sh`

Observed failure remains the same read-only `cargo pgrx install --test`
destination:

- `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
- `Read-only file system (os error 30)`

## Outcome

The pg tests now describe the runtime the branch actually has:

1. source-backed exact-score checks use source-backed truth
2. binary-lane tests run on a binary-capable fixture
3. live-window assertions stop demanding movement where only set-level parity is
   required
4. round-trip/vacuum coverage follows the current fixture ranking instead of old
   hard-coded ids

## Next Slice

Clean up the remaining low-level build/study assumptions that were exposed while
rerunning these repaired lanes:

1. the HNSW self-score offset on QJL-enabled 4-bit scalar builds
2. the grouped-PQ nibble-pack study fixture that still assumed a smaller
   centroid table than the current 4-bit lane uses
