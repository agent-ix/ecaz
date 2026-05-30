# Task 67 Packet 010 Artifact Manifest

- head SHA: `0471c4cfe2f738ee7353cc5bd99a01bef289d6e1`
- task bucket: `reviews/task-67/`
- packet path: `reviews/task-67/010-intel-measurement-suite/`
- timestamp: `2026-05-30T02:07:38Z`
- lane: Intel measurement suite preparation
- fixture: `target/real-corpus/staged-task50/ec_real_100k_*`
- storage format: `rabitq`
- rerank mode: `heap_f32`, plus sidecar rerank variants
  `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- surface: `ec_ivf`, `quant_bits=1`, `nlists=128`, `nprobe=128`
- isolated one-index-per-table or shared-table surfaces: isolated prefixes
  `task67_intel_100k_rabitq1_scalar` and
  `task67_intel_100k_rabitq1_auto`

## Artifacts

### `task67-intel-suite.json`

- command used to validate:
  `target/debug/ecaz bench suite audit --config reviews/task-67/010-intel-measurement-suite/artifacts/task67-intel-suite.json`
- result: passed
- key line:
  `[suite:task67-intel-rabitq-simd-gate] audit passed: 10 steps`

### `suite-manifest.json`

- command used to generate:
  `target/debug/ecaz bench suite run --config reviews/task-67/010-intel-measurement-suite/artifacts/task67-intel-suite.json --dry-run --manifest-output reviews/task-67/010-intel-measurement-suite/artifacts/suite-manifest.json`
- result: passed
- key line:
  `[suite:task67-intel-rabitq-simd-gate] wrote reviews/task-67/010-intel-measurement-suite/artifacts/suite-manifest.json`

### `validation.log`

- command:
  `target/debug/ecaz bench suite status --manifest reviews/task-67/010-intel-measurement-suite/artifacts/suite-manifest.json`
- result: passed
- key line:
  `[suite:task67-intel-rabitq-simd-gate] completed=0 failed=0 skipped=0 dry_run=10 missing_artifacts=0 stale=0`

This packet intentionally contains dry-run evidence only. The final Task 67
measurement packet must execute the suite on the Intel benchmark host.
