# Artifact Manifest

- Head SHA: `dd42450f7fd0215d9c7385dd9cc1b25c0443b769`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/004-frontier-neighbor-retention/`
- Timestamp: `2026-05-31T18:51:14Z`
- Lane / fixture / storage format / rerank mode: M5 local real10K DBpedia; `ec_diskann`; `pq_fastscan`; exact heap rerank with `rerank_budget=64`.
- Isolated one-index-per-table or shared-table surface: isolated one-index-per-table surface using prefix `task70_004_diskann`.

## Artifacts

| Artifact | Description |
| --- | --- |
| `suite.json` | Packet-local `ecaz bench suite` config for the post-slice real10K DiskANN L=64/L=200 measurement. |
| `suite-dry-run.log` / `suite-dry-run-manifest.json` | Dry-run expansion and structured manifest. |
| `suite-run.log` / `suite-manifest.json` | Full suite run log and structured manifest. |
| `results.jsonl` | Normalized suite results for load, recall, latency, EXPLAIN snapshots, and pgvectorscale comparison. |
| `load-diskann-real10k.log` | Isolated corpus/index load log. |
| `recall-diskann-real10k-l64-l200.log` | Recall at L=64 and L=200. |
| `latency-diskann-real10k-l64-l200-profiled.log` | Latency at L=64 and L=200. |
| `profile-notices-diskann-real10k-l64.sql` / `profile-notices-diskann-real10k-l64.log` | Raw suite step and 200 scan profile NOTICE rows at L=64. |
| `profile-notices-diskann-real10k-l200.sql` / `profile-notices-diskann-real10k-l200.log` | Raw suite step and 200 scan profile NOTICE rows at L=200. |
| `explain-diskann-real10k-l64.sql` / `explain-diskann-real10k-l64.log` | Representative low-L EXPLAIN evidence. |
| `explain-diskann-real10k-l200.sql` / `explain-diskann-real10k-l200.log` | Representative high-L EXPLAIN evidence. |
| `compare-vectorscale-real10k-l64-l200.log` | pgvectorscale comparison. |
| `truth-real10k-k10.json` | Ground-truth cache. |
| `phase2-frontier-summary.md` | Reduced results and interpretation for this slice. |
| `cargo-test-diskann-scan.log` | Focused scan unit test log. |
| `cargo-check-pg18.log` | PG18 compile check log. |
| `install-ecaz-pg-test.log` | Extension install log for the current build. |

## Commands

```sh
cargo fmt --check
cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::
cargo check --all-targets --no-default-features --features pg18
./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/004-frontier-neighbor-retention/artifacts/install-ecaz-pg-test.log
./target/debug/ecaz bench suite run --config reviews/task-70/004-frontier-neighbor-retention/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/004-frontier-neighbor-retention/artifacts/suite-manifest.json --results-output reviews/task-70/004-frontier-neighbor-retention/artifacts/results.jsonl --log-file reviews/task-70/004-frontier-neighbor-retention/artifacts/suite-run.log
```

## Key Results

- Recall floor preserved: L=64 recall@10 `0.9965`; L=200 recall@10 `0.9975`.
- Latency: L=64 mean `0.64 ms`, p95 `0.73 ms`; L=200 mean `0.91 ms`, p95 `1.10 ms`.
- pgvectorscale comparison: L=64 `ec_diskann` mean `0.60 ms` vs pgvectorscale `0.60 ms`; L=200 `ec_diskann` mean `0.77 ms` vs pgvectorscale `1.13 ms`.
- Phase split: frontier mean `269.62 -> 263.60 us` at L=64; `553.04 -> 527.90 us` at L=200.
