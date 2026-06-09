# Manifest: Task 92 Packet 014 Off-Path Calibration Run

- Head SHA: `86f21e7188f387321959b4b78f9b318778df681f`
- Task bucket: `reviews/task-92/`
- Packet path: `reviews/task-92/014-offpath-calibration-run/`
- Lane: local PG18 pgrx fixture
- Host/socket: `/home/peter/.pgrx`
- Port: `28818`
- Database: `postgres`
- Fixture: synthetic 4096-row / 64-query 1536-dim SPIRE TurboQuant
- Storage format: `turboquant`
- Rerank mode: not applicable
- Isolation: local one-index SPIRE fixture loaded under prefix `task92_offpath_spire_turboquant`
- Graviton 4 status: not Graviton 4 evidence; final closeout still needs the standard Graviton 4 lane with measured SVE2 vector length

## Artifacts

### `generate-corpus.log`

- Command: `target/debug/ecaz corpus generate --output reviews/task-92/014-offpath-calibration-run/artifacts/task92_offpath_spire_turboquant_corpus.tsv --n 4096 --dim 1536 --seed 42 --kind corpus --log-file reviews/task-92/014-offpath-calibration-run/artifacts/generate-corpus.log`
- Result: generated 4096 corpus rows.
- The generated TSV was intentionally not committed because it is reproducible and about 60 MB with the query TSV.

### `generate-queries.log`

- Command: `target/debug/ecaz corpus generate --output reviews/task-92/014-offpath-calibration-run/artifacts/task92_offpath_spire_turboquant_queries.tsv --n 64 --dim 1536 --seed 4242 --kind queries --log-file reviews/task-92/014-offpath-calibration-run/artifacts/generate-queries.log`
- Result: generated 64 query rows.
- The generated TSV was intentionally not committed; `load-spire-turboquant.log` records its SHA-256 digest.

### `load-spire-turboquant.log`

- Command: `target/debug/ecaz corpus load --prefix task92_offpath_spire_turboquant --corpus-file reviews/task-92/014-offpath-calibration-run/artifacts/task92_offpath_spire_turboquant_corpus.tsv --queries-file reviews/task-92/014-offpath-calibration-run/artifacts/task92_offpath_spire_turboquant_queries.tsv --profile ec_spire --storage-format turboquant --dim 1536 --bits 4 --seed 42 --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-92/014-offpath-calibration-run/artifacts/load-spire-turboquant.log`
- Key lines:
  - corpus: 4096 rows, SHA-256 `2a15917d2f1469fac3660a896441f32b3881d4b711f4377171ff0098ba1273d5`
  - queries: 64 rows, SHA-256 `e6c5a265c98948a3db70ae59c5b1cedba09ffe85ad9d32113dd0e19682979e02`
  - index: `task92_offpath_spire_turboquant_turboquant_idx [storage_format=turboquant]`

### `suite-run.log`

- Command: `target/debug/ecaz bench suite run --config crates/ecaz-cli/suites/task92-offpath-calibration.json --artifact-dir reviews/task-92/014-offpath-calibration-run/artifacts --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-92/014-offpath-calibration-run/artifacts/suite-manifest.json --results-output reviews/task-92/014-offpath-calibration-run/artifacts/results.jsonl --log-file reviews/task-92/014-offpath-calibration-run/artifacts/suite-run.log`
- Result: suite completed and wrote `results.jsonl`.

### `suite-manifest.json`

- Structured manifest emitted by `ecaz bench suite run`.
- Confirms the kernel-off cell used `--session-guc ec_spire.candidate_batch_scoring=off`.

### `results.jsonl`

- Normalized result rows emitted by `ecaz bench suite run`.
- Kernel-on wall latency row:
  - mean `438.3 ms`
  - p50 `435.2 ms`
  - p95 `458.5 ms`
  - p99 `472.6 ms`
- Kernel-off wall latency row:
  - mean `440.9 ms`
  - p50 `438.4 ms`
  - p95 `455.9 ms`
  - p99 `472.3 ms`

### `latency-spire-turboquant-lut32-kernel-on.log`

- Kernel-on result and counter log.
- Key counter line:
  - `[task87-counters] command=latency label=nprobe=32 surface=spire flushes=1024 candidates=65453 elapsed_nanos=840868757 elapsed_ms=840.868757 lut32_flushes=1024 lut32_candidates=49024`

### `latency-spire-turboquant-lut32-kernel-off.log`

- Kernel-off result and counter log.
- Key counter line:
  - `[task87-counters] command=latency label=nprobe=32 surface=spire flushes=1024 candidates=65453 elapsed_nanos=952817421 elapsed_ms=952.817421 lut32_flushes=0 lut32_candidates=0`

## Interpretation

The local synthetic run validates the off-path toggle shape:

- total SPIRE flushes and candidates are identical across kernel-on/off;
- kernel attribution is nonzero when enabled;
- kernel attribution drops to zero when `ec_spire.candidate_batch_scoring=off`;
- wall mean drift is `+0.59%`, p50 drift is `+0.74%`, p95 drift is `-0.57%`, and p99 drift is `-0.06%`.

This packet does not close Task 92. The final acceptance packet still needs
standard-corpus Graviton 4 evidence and measured SVE2 vector-length reporting.
