# Task 70 / Packet 003: Phase 1 Suite Config

## Packet Scope

- Head: `965af31f1f39ced34a921a41a663bf63dd5de13d`
- Artifact config: `artifacts/suite.json`
- Dry-run manifest: `artifacts/suite-dry-run-manifest.json`
- Dry-run log: `artifacts/suite-dry-run.log`

This packet adds the packet-local `ecaz bench suite` config for Task 70 Phase 1. It does not claim Phase 1 measurements are complete; it prepares the canonical runner surface for the actual M5 real10K profiling run.

## Why

Task 70 requires the real10K DiskANN scan split at L=64 and L=200, plus recall and pgvectorscale comparison evidence. The repo rules require benchmark matrices and multi-step measurement runs to be driven by `ecaz bench suite` with a checked-in `SuiteConfig`. This packet provides that config after packets 001 and 002 made scan profile NOTICE output suite-addressable.

## Suite Shape

The suite uses isolated `task70_phase1_real10k_diskann` tables and the existing staged real10K inputs:

- load `ec_diskann` with `graph_degree=32`, `build_list_size=100`, `alpha=1.2`, `rerank_budget=64`, `top_k=10`
- recall at `list_size` 64 and 200
- latency at `list_size` 64 and 200 with `session_gucs: ["ec_diskann.scan_profile_notice=on"]`
- EXPLAIN at L=64 and L=200
- pgvectorscale comparison at L=64 and L=200

## Validation

Dry-run command:

```sh
cargo run -p ecaz-cli -- bench suite run --config reviews/task-70/003-phase1-suite-config/artifacts/suite.json --dry-run --manifest-output reviews/task-70/003-phase1-suite-config/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/003-phase1-suite-config/artifacts/suite-dry-run.log
```

Key dry-run line:

```text
latency-diskann-real10k-l64-l200-profiled -> ... bench latency ... --sweep "64,200" ... --session-guc ec_diskann.scan_profile_notice=on ...
```

## Next Step

Run this suite after installing the current extension build into PG18, then promote the measured logs into a Phase 1 characterization packet with ranking of P0 scan-kernel slices.
