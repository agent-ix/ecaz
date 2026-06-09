# Task 92 Packet 014: Off-Path Calibration Run

## Summary

This checkpoint runs the Task 92 off-path calibration suite from Packet 013
against a local PG18 SPIRE TurboQuant fixture.

It is deliberately scoped as local calibration evidence, not the final
Graviton 4 closeout gate. The final closeout still needs the standard prepared
corpus on the Graviton 4 lane and measured SVE2 vector-length evidence.

## What Changed

- Generated a deterministic synthetic 4096-row / 64-query 1536-dim fixture.
- Loaded it with `ecaz corpus load --profile ec_spire --storage-format turboquant`.
- Ran `crates/ecaz-cli/suites/task92-offpath-calibration.json` with:
  - kernel-on cell;
  - kernel-off cell using `ec_spire.candidate_batch_scoring=off`;
  - Task 87 candidate-batch counter collection enabled for both cells.

The large generated TSV payloads were not committed; the packet keeps the
generation logs, load log, suite manifest, result rows, and latency/counter
logs. The load log records the row counts and SHA-256 digests.

## Key Results

- Kernel-on SPIRE counters:
  - `flushes=1024`
  - `candidates=65453`
  - `elapsed_nanos=840868757`
  - `lut32_flushes=1024`
  - `lut32_candidates=49024`
- Kernel-off SPIRE counters:
  - `flushes=1024`
  - `candidates=65453`
  - `elapsed_nanos=952817421`
  - `lut32_flushes=0`
  - `lut32_candidates=0`
- Wall latency:
  - mean: `438.3 ms` on vs `440.9 ms` off (`+0.59%`)
  - p50: `435.2 ms` on vs `438.4 ms` off (`+0.74%`)
  - p95: `458.5 ms` on vs `455.9 ms` off (`-0.57%`)
  - p99: `472.6 ms` on vs `472.3 ms` off (`-0.06%`)

This proves the suite-level kernel-off GUC reaches the intended SPIRE scoring
path on a loaded fixture: the same total flush/candidate count is observed in
both cells, while LUT32 kernel attribution drops to zero in the kernel-off
cell.

## Validation

Artifacts are under `reviews/task-92/014-offpath-calibration-run/artifacts/`.

- `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --sql "SELECT 1 AS ok"`
- `target/debug/ecaz corpus generate --output ...task92_offpath_spire_turboquant_corpus.tsv --n 4096 --dim 1536 --seed 42 --kind corpus --log-file ...generate-corpus.log`
- `target/debug/ecaz corpus generate --output ...task92_offpath_spire_turboquant_queries.tsv --n 64 --dim 1536 --seed 4242 --kind queries --log-file ...generate-queries.log`
- `target/debug/ecaz corpus load --prefix task92_offpath_spire_turboquant --profile ec_spire --storage-format turboquant --dim 1536 --bits 4 --seed 42 --database postgres --host /home/peter/.pgrx --port 28818 --log-file ...load-spire-turboquant.log`
- `target/debug/ecaz bench suite run --config crates/ecaz-cli/suites/task92-offpath-calibration.json --artifact-dir reviews/task-92/014-offpath-calibration-run/artifacts --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output ...suite-manifest.json --results-output ...results.jsonl --log-file ...suite-run.log`

## Review Focus

- Confirm this packet is acceptable as local calibration smoke for the Packet
  013 suite shape.
- Confirm the final Task 92 closeout should still require the standard
  Graviton 4 lane, SVE2 dispatch evidence, and measured runtime vector length.
