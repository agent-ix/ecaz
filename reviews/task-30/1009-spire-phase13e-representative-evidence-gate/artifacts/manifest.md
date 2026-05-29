# SPIRE Phase 13e Representative Evidence Gate Manifest

- Head SHA: `a8544bafb3060973615a72143a3b54578424c93b`
- Task bucket: `reviews/task-30/1009-spire-phase13e-representative-evidence-gate`
- Lane: SPIRE Phase 13e representative latency, recall, and pooling evidence gate
- Fixture: packet-local TSV summaries copied from packet `1004` sample output, plus a negative pooling-latency delta fixture
- Storage format: summary TSV artifacts
- Rerank mode: not applicable; verifier only validates summary evidence shape and pooled-vs-unpooled deltas
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `bash-n-verifier.log`

- Command: `bash -n scripts/spire-aws/verify-representative-performance-summary.sh`
- Timestamp: `2026-05-27 08:44:34-07:00`
- Key result: shell syntax validation passed.

### `verify-good-summary.log`

- Command: `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1009-spire-phase13e-representative-evidence-gate/artifacts/good-summary`
- Timestamp: `2026-05-27 08:44:34-07:00`
- Key result: `representative performance summary verified`.

### `verify-bad-summary.log`

- Command: `scripts/spire-aws/verify-representative-performance-summary.sh reviews/task-30/1009-spire-phase13e-representative-evidence-gate/artifacts/bad-summary`
- Timestamp: `2026-05-27 08:44:34-07:00`
- Expected result: exit code `2`.
- Key result: `ERROR: pooling delta improvement row missing or incomplete`.

### `make-n-verify-summary.log`

- Command: `make -C infra/spire-aws -n verify-representative-performance-summary ARTIFACT_DIR=reviews/task-30/1009-spire-phase13e-representative-evidence-gate/artifacts/good-summary`
- Timestamp: `2026-05-27 08:44:34-07:00`
- Key result: dry-run target invokes `scripts/spire-aws/verify-representative-performance-summary.sh`.

### `make-preflight.log`

- Command: `make -C infra/spire-aws preflight`
- Timestamp: `2026-05-27 08:44:34-07:00`
- Key result lines:
  - `Terraform has been successfully initialized!`
  - `Success! The configuration is valid.`
  - `shellcheck not found; skipping shellcheck`

### `good-summary/`

- Command: copied from the packet `1004` representative priority sample output.
- Key result: includes latency p50/p95/p99, recall@k, production profile rows, disabled/enabled pooling rows, and positive socket-open plus latency-p95 deltas with zero recall delta.

### `bad-summary/`

- Command: copied from `good-summary/` with `representative-pooling-delta-summary.tsv` changed to keep latency p95 flat.
- Key result: verifier rejects the fixture because pooling does not improve latency p95.
