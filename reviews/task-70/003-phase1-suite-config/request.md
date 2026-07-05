# Task 70 / Packet 003: Phase 1 Suite Config + First Run

## Packet Scope

- Head: `26cc6d9de31e99fd8349df1f64cb11132c468eac`
- Artifact config: `artifacts/suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Normalized results: `artifacts/results.jsonl`
- Phase summary: `artifacts/phase1-profile-summary.md`

This packet now includes the packet-local `ecaz bench suite` config and an M5 local first run for Task 70 Phase 1. It asks for review of the measurement shape and the ranked P0 list; Phase 1 is still open until reviewer feedback is processed.

## Why

Task 70 requires the real10K DiskANN scan split at L=64 and L=200, plus recall and pgvectorscale comparison evidence. The repo rules require benchmark matrices and multi-step measurement runs to be driven by `ecaz bench suite` with a checked-in `SuiteConfig`. This packet provides that config and the first complete suite output after packets 001 and 002 made scan profile NOTICE output suite-addressable.

## Suite Shape

The suite uses isolated `task70_phase1_real10k_diskann` tables and the existing staged real10K inputs. In this checkout, those real10K DBpedia inputs live under the existing `ec_hnsw_real_10k_*` filenames in `data/task31_m5_dbpedia_staged/`; the suite builds an `ec_diskann` index over that corpus.

- load `ec_diskann` with `graph_degree=32`, `build_list_size=100`, `alpha=1.2`, `rerank_budget=64`, `top_k=10`
- recall at `list_size` 64 and 200
- latency at `list_size` 64 and 200 with `session_gucs: ["ec_diskann.scan_profile_notice=on"]`
- raw `dev sql` profile steps at L=64 and L=200 that run 200 indexed queries and capture one `ec_diskann_scan_profile` NOTICE per query
- EXPLAIN at L=64 and L=200
- pgvectorscale comparison at L=64 and L=200

## Validation

Dry-run command:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/003-phase1-suite-config/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/003-phase1-suite-config/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/003-phase1-suite-config/artifacts/suite-dry-run.log
```

Full run command:

```sh
./target/debug/ecaz bench suite run --config reviews/task-70/003-phase1-suite-config/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/003-phase1-suite-config/artifacts/suite-manifest.json --results-output reviews/task-70/003-phase1-suite-config/artifacts/results.jsonl --log-file reviews/task-70/003-phase1-suite-config/artifacts/suite-run.log
```

Key dry-run line:

```text
latency-diskann-real10k-l64-l200-profiled -> ... bench latency ... --sweep "64,200" ... --session-guc ec_diskann.scan_profile_notice=on ...
```

## Key Results

- Recall floor preserved: L=64 recall@10 `0.9965`; L=200 recall@10 `0.9975`.
- Latency: L=64 mean `0.65 ms`, p50 `0.64 ms`, p95 `0.75 ms`; L=200 mean `0.96 ms`, p50 `0.95 ms`, p95 `1.18 ms`.
- pgvectorscale comparison: at L=64, `ec_diskann` mean `0.64 ms` vs pgvectorscale `0.60 ms`; at L=200, `ec_diskann` mean `0.83 ms` vs pgvectorscale `1.13 ms`.
- Phase split from 200 NOTICE rows per L: frontier maintenance dominates (`72.94%` at L=64, `83.61%` at L=200), followed by exact heap rerank (`21.83%` at L=64, `13.20%` at L=200).

## Proposed P0 Ranking

1. Frontier / candidate management.
2. Exact heap rerank fetch/detoast.

The summary shelves graph read/decode cache, binary sidecar prefilter tuning, and result materialization for this fixture because each measured below the task's P0 threshold.
