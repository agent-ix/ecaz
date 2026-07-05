# Task 65b Packet 014 Artifact Manifest

- head SHA: `7034ac5fcee68815a467674d130d6ab65372b52b`
- task bucket: `reviews/task-65b/014-batched-backlink-reducer`
- timestamp: `2026-06-05T18:40:00Z`
- lane: local PG18, `ec_diskann`, `pq_fastscan`, real10k/real100k DBpedia fixtures
- storage format: `pq_fastscan`
- rerank mode: default `ec_diskann` recall sweep (`list_size=64,128,200`)
- isolation: one index per packet-local table prefix
- installed backend SHA: `a0205bf54f580a35f5638a73857727435004490b4409f6031a8c2f6a3e5ad060`

## Code Validation

| artifact | command | result |
|---|---|---|
| `cargo-fmt-check-final.log` | `cargo fmt --check` | passed |
| `cargo-check-pg18-after-final-alpha.log` | `cargo check -p ecaz --lib --no-default-features --features pg18` | passed |
| `cargo-test-vamana-task65b-after-final-alpha.log` | `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_` | passed, 5 tests |
| `cargo-test-build-task65b-after-final-alpha.log` | `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b_` | passed, 5 tests |

Latest source validation after `7034ac5fc`:

- `cargo fmt --check`: passed.
- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_`: passed, 5 tests.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b_`: passed, 5 tests.

## Parallel Backlink Real10k B64 Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real10k-extra-batch-suite.json --host /Users/peter/.pgrx --port 28818 --only load-real10k-w8-b64 --only recall-real10k-w8-b64 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b64-after-parallel-backlink-plan-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b64-after-parallel-backlink-plan-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/b64-after-parallel-backlink-plan-run.log`

Key result lines:

- `load-real10k-w8-b64`: `parallel_effective_workers=8`, `parallel_batch_size=64`, `total_ms=1080`, `core_graph_ms=905`, `parallel_proposal_ms=492`, `parallel_reducer_ms=256`, `parallel_epochs=157`.
- `recall-real10k-w8-b64`, `list_size=200`: `recall@10=0.9950`, CI95 `0.9908..0.9973`, mean query time `0.81 ms`.
- Gate status: real10k build time passes `<=3s`; recall passes the Task 65 floor threshold of `>=0.9925`.

## Final-Alpha Real10k B96 Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real10k-mid-batch-suite.json --host /Users/peter/.pgrx --port 28818 --only load-real10k-w8-b96 --only recall-real10k-w8-b96 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b96-after-final-alpha-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b96-after-final-alpha-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/b96-after-final-alpha-run.log`

Key result lines:

- `load-real10k-w8-b96`: `parallel_effective_workers=8`, `parallel_batch_size=96`, `total_ms=2737`, `core_graph_ms=2557`, `parallel_proposal_ms=510`, `parallel_reducer_ms=1928`, `parallel_epochs=105`.
- `recall-real10k-w8-b96`, `list_size=200`: `recall@10=0.9920`, CI95 `0.9870..0.9951`, mean query time `0.82 ms`.
- Gate status: build time passes; recall misses the Task 65 floor threshold by `0.0005`.

## Parallel Backlink Real100k B64 Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real100k-b64-suite.json --host /Users/peter/.pgrx --port 28818 --only load-real100k-w8-b64 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b64-after-parallel-backlink-plan-load-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b64-after-parallel-backlink-plan-load-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b64-after-parallel-backlink-plan-load-run.log`

Key result lines:

- `load-real100k-w8-b64`: `parallel_effective_workers=8`, `parallel_batch_size=64`, `total_ms=36490`, `core_graph_ms=35701`, `parallel_proposal_ms=14003`, `parallel_reducer_ms=19115`, `parallel_epochs=1563`.
- `recall-real100k-w8-b64`, `list_size=200`: not rerun after `7034ac5fc`; prior packet-local supporting row was `recall@10=0.9750`.
- Gate status: build time still fails the `<=30s` Task 65b gate.

## Parallel Backlink Real100k B768 Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real100k-b768-suite.json --host /Users/peter/.pgrx --port 28818 --only load-real100k-w8-b768 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b768-after-effective-workers-load-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b768-after-effective-workers-load-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b768-after-effective-workers-load-run.log`

Recall command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real100k-b768-suite.json --host /Users/peter/.pgrx --port 28818 --only recall-real100k-w8-b768 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b768-after-effective-workers-recall-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b768-after-effective-workers-recall-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b768-after-effective-workers-recall-run.log`

Key result lines:

- `load-real100k-w8-b768`: `parallel_requested_workers=8`, `parallel_effective_workers=8`, `parallel_batch_size=768`, `total_ms=29771`, `core_graph_ms=28473`, `parallel_proposal_ms=13730`, `parallel_reducer_ms=14475`, `parallel_epochs=131`.
- `recall-real100k-w8-b768`, `list_size=200`: `recall@10=0.9700`, CI95 `0.9616..0.9766`, mean query time `1.35 ms`.
- Gate status: real100k build time passes `<=30s`; recall is exactly 0.5pp below the `w8/b64` supporting recall row (`0.9750`).

## Exploratory Supporting Artifacts

- `real10k-probe-rerun-results.jsonl`: pre-final-alpha b16/b32 comparison after batched backlink reducer.
- `extra-batch-results.jsonl`: pre-final-alpha b64/b128 comparison.
- `mid-batch-results.jsonl`: pre-final-alpha b96/b112 comparison.
- `b128-after-prune-results.jsonl`: isolated b128 run after the prune dominance cleanup.
- `b64-after-arc-batch-results.jsonl`: b64 run after adjacency Arc batching, before final-alpha pruning.
