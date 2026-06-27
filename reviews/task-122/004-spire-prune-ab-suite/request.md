# Task 122 review request: SPIRE prune A/B suite

## Scope

This checkpoint makes the SPIRE pre-materialization prune measurable with a
same-binary A/B switch and adds the Task 122 suite packet for 10k / 50k / 100k
TQ-vs-RaBitQ comparison.

Code commit under review:

- `aa799704b` `Gate SPIRE pre-materialization prune`

Packet artifacts:

- `artifacts/task122-spire-prune-ab-suite.json`
- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-run-10k-debug.log`
- `artifacts/suite/suite-manifest.json`
- `artifacts/suite/results.jsonl`
- per-step logs under `artifacts/suite/`

## What changed

- Added `ec_spire.pre_materialization_prune`, default `on`.
- Gated both SPIRE pre-materialization threshold checks behind that GUC.
- Added a suite config with:
  - TQ prune-on/off recall, latency, and SPIRE pipeline steps
  - TQ storage
  - RaBitQ recall, latency, and storage comparator
  - 10k / 50k / 100k staged corpus coverage

## Validation

Static / focused Rust validation passed:

- `cargo fmt --check`
- `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics`
- `cargo check -p ecaz --lib --no-default-features --features pg18`

Suite harness validation passed:

- `ecaz bench suite audit --config ...`
- `ecaz bench suite run --config ... --dry-run ...`

Local PG18 GUC check showed:

- `ecaz_build_profile()` = `debug`
- `ec_spire.pre_materialization_prune` = `on`

## Debug smoke result

I ran the 10k slice with `--allow-debug-backend` to verify the suite and GUC
plumbing. This is not decision-grade closeout evidence.

Key 10k smoke lines from `artifacts/suite/results.jsonl`:

- TQ prune on: recall@k `1.0000`, latency p50 `92.4 ms`, p95 `96.1 ms`
- TQ prune off: recall@k `1.0000`, latency p50 `93.8 ms`, p95 `99.0 ms`
- Candidate materialization dropped from `251,555` to `8,495` with pruning on.
- Heap rerank rows stayed at `2,500` in both modes.
- RaBitQ comparator: recall@k `1.0000`, latency p50 `74.2 ms`, p95 `81.3 ms`
- Storage: TQ SPIRE index `8.9 MiB`; RaBitQ SPIRE index `9.0 MiB`

## Reviewer notes

Please review the GUC gating and suite shape. I am not claiming Task 122
closeout from this packet; release A/B evidence at 10k / 50k / 100k is still
required before any closeout decision.
