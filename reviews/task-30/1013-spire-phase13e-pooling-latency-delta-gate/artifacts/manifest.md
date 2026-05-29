# SPIRE Phase 13e Pooling Latency Delta Gate Manifest

- Head SHA: `7cd7ba73fb20659be7f40b9e101eacfd643862b0`
- Task bucket: `reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate`
- Lane: SPIRE Phase 13e representative performance readiness
- Fixture: packet-local representative suite sample output
- Storage format: not applicable; summary/verifier gate only
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Timestamp: `2026-05-27T16:11:58Z`

## Artifacts

### `bash-n-summary-verifier.log`

- Command: `bash -n scripts/spire-aws/summarize-representative-performance.sh scripts/spire-aws/verify-representative-performance-summary.sh`
- Result: shell syntax validation passed.

### `sample-input/`

- Command: copied from the previously reviewed endpoint-identity representative sample input.
- Purpose: stable local fixture containing representative latency, recall, production profile, and pooling A/B JSONL rows.

### `summarize-pooling-latency-sample.log`

- Command: `scripts/spire-aws/summarize-representative-performance.sh reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate/artifacts/sample-input reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate/artifacts/sample-output`
- Result: emitted latency/recall, production-profile, pooling-comparison, and pooling-delta summary TSV files.

### `sample-output/representative-pooling-delta-summary.tsv`

- Key result: pooled-vs-unpooled row includes positive socket-open reduction and positive latency deltas for p50, p95, and p99 with zero recall regression.
- Sample row:
  - `nprobe=8`
  - `socket_open_delta=996`
  - `latency_p50_delta_ms=1`
  - `latency_p95_delta_ms=4`
  - `latency_p99_delta_ms=6`
  - `recall_delta=0`

### `verify-good-pooling-latency-summary.log`

- Command: `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate/artifacts/sample-output`
- Key result: `representative performance summary verified`.

### `bad-summary/`

- Command: copied from `sample-output/` with `representative-pooling-delta-summary.tsv` modified so `latency_p99_delta_ms=0`.
- Purpose: proves the representative verifier rejects incomplete p50/p95/p99 pooling latency improvement evidence.

### `verify-bad-p99-delta-summary.log`

- Command: `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1013-spire-phase13e-pooling-latency-delta-gate/artifacts/bad-summary`
- Expected result: exit code `2`.
- Key result: `ERROR: pooling delta improvement row missing or incomplete`.

### `preflight-representative-performance.log`

- Command: `scripts/spire-aws/preflight-representative-performance.sh`
- Key result: `SPIRE representative performance preflight passed`.

### `make-preflight-representative-performance.log`

- Command: `make -C infra/spire-aws preflight-representative-performance`
- Key result: Make target invokes the representative performance preflight and passes.
