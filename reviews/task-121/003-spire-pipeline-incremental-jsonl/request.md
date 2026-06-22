# Task 121 Review Request - Incremental SPIRE Pipeline JSONL Outputs

## Scope

This packet reviews commit `64d7d1f843269434de5f651452a3c79a26518290`
(`Flush SPIRE pipeline JSONL outputs incrementally`).

The change makes `bench spire-pipeline` flush requested JSONL artifacts after
each query, using the rows accumulated so far, and again at command completion.
It does not change query execution, metrics, routing, recall, or report
rendering.

## Why

Task 121 Stage 1 needs q200 multi-sweep pipeline runs. The first q200
baseline attempt loaded cached truth successfully, but spent more than 33
minutes in query-metrics KNN work without writing funnel/stage artifacts. If a
long run has to be canceled, the old behavior left no partial route-containment
evidence. The new behavior makes long local-only runs inspectable and
recoverable without introducing bespoke sweep scripts.

## Validation

- `cargo test -p ecaz-cli commands::bench::spire_pipeline`: 19 passed.
- `cargo build -p ecaz-cli --bin ecaz`: passed with the pre-existing
  `LoadedDistributedPlacementConfig.path` dead-code warning.
- Local q20/nprobe96 suite smoke under this packet's `--artifact-dir`:
  - During the active pipeline run, before process completion, the packet had
    non-empty JSONL artifacts:
    - `pipeline-baseline-q20-n96-funnel.jsonl`: 29,461 bytes
    - `pipeline-baseline-q20-n96-stage-containment.jsonl`: 50,536 bytes
  - Final q20 pipeline result remained sane:
    - recall@10 1.0000
    - p50 3248.748 ms
    - p95 3316.355 ms
    - candidate_sum / heap_rerank_sum 1,522,002

## Artifacts

See `artifacts/manifest.md`.

Truth-cache JSON is not committed; it is regenerable cache state. The suite
manifests, logs, results, funnel JSONL, and stage-containment JSONL are the
durable evidence.
