# Task 65b Packet 014 Artifact Manifest

- head SHA: `f3809bf18e26df1d420d35693c3960ef2567c34e`
- task bucket: `reviews/task-65b/014-batched-backlink-reducer`
- timestamp: `2026-06-05T15:19:20Z`
- lane: local PG18, `ec_diskann`, `pq_fastscan`, real10k/real100k DBpedia fixtures
- storage format: `pq_fastscan`
- rerank mode: default `ec_diskann` recall sweep (`list_size=64,128,200`)
- isolation: one index per packet-local table prefix
- installed backend SHA: `8197fd9a40a5b3ccdb04330deaa37d602fe58a31332ecf26b5dc96ca2e709a0a`

## Code Validation

| artifact | command | result |
|---|---|---|
| `cargo-fmt-check-final.log` | `cargo fmt --check` | passed |
| `cargo-check-pg18-after-final-alpha.log` | `cargo check -p ecaz --lib --no-default-features --features pg18` | passed |
| `cargo-test-vamana-task65b-after-final-alpha.log` | `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_` | passed, 5 tests |
| `cargo-test-build-task65b-after-final-alpha.log` | `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b_` | passed, 5 tests |

## Final-Alpha Real10k Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real10k-extra-batch-suite.json --host /Users/peter/.pgrx --port 28818 --only load-real10k-w8-b64 --only recall-real10k-w8-b64 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b64-after-final-alpha-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b64-after-final-alpha-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/b64-after-final-alpha-run.log`

Key result lines:

- `load-real10k-w8-b64`: `parallel_effective_workers=8`, `parallel_batch_size=64`, `total_ms=2873`, `core_graph_ms=2709`, `parallel_proposal_ms=502`, `parallel_reducer_ms=2041`, `parallel_epochs=157`.
- `recall-real10k-w8-b64`, `list_size=200`: `recall@10=0.9950`, CI95 `0.9908..0.9973`, mean query time `0.80 ms`.
- Gate status: real10k build time passes `<=3s`; recall passes the Task 65 floor threshold of `>=0.9925`.

## Final-Alpha Real10k B96 Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real10k-mid-batch-suite.json --host /Users/peter/.pgrx --port 28818 --only load-real10k-w8-b96 --only recall-real10k-w8-b96 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b96-after-final-alpha-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/b96-after-final-alpha-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/b96-after-final-alpha-run.log`

Key result lines:

- `load-real10k-w8-b96`: `parallel_effective_workers=8`, `parallel_batch_size=96`, `total_ms=2737`, `core_graph_ms=2557`, `parallel_proposal_ms=510`, `parallel_reducer_ms=1928`, `parallel_epochs=105`.
- `recall-real10k-w8-b96`, `list_size=200`: `recall@10=0.9920`, CI95 `0.9870..0.9951`, mean query time `0.82 ms`.
- Gate status: build time passes; recall misses the Task 65 floor threshold by `0.0005`.

## Final-Alpha Real100k B64 Probe

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/014-batched-backlink-reducer/real100k-b64-suite.json --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b64-after-final-alpha-manifest.json --results-output reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b64-after-final-alpha-results.jsonl --log-file reviews/task-65b/014-batched-backlink-reducer/artifacts/real100k-b64-after-final-alpha-run.log`

Key result lines:

- `load-real100k-w8-b64`: `parallel_effective_workers=8`, `parallel_batch_size=64`, `total_ms=139356`, `core_graph_ms=138567`, `parallel_proposal_ms=13862`, `parallel_reducer_ms=122176`, `parallel_epochs=1563`.
- `recall-real100k-w8-b64`, `list_size=200`: `recall@10=0.9750`, CI95 `0.9672..0.9810`, mean query time `1.41 ms`.
- Gate status: recall holds the packet 001/013 real100k floor, but build time fails the `<=30s` Task 65b gate.

## Exploratory Supporting Artifacts

- `real10k-probe-rerun-results.jsonl`: pre-final-alpha b16/b32 comparison after batched backlink reducer.
- `extra-batch-results.jsonl`: pre-final-alpha b64/b128 comparison.
- `mid-batch-results.jsonl`: pre-final-alpha b96/b112 comparison.
- `b128-after-prune-results.jsonl`: isolated b128 run after the prune dominance cleanup.
- `b64-after-arc-batch-results.jsonl`: b64 run after adjacency Arc batching, before final-alpha pruning.
