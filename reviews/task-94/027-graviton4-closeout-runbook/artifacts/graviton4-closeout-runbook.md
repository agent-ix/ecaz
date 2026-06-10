# Task 94 Graviton 4 Closeout Runbook

This runbook prepares the approved-AWS evidence pass for Task 94. It is not an
execution log. Do not run these commands until the project owner explicitly
approves Graviton 4/AWS testing for Task 94.

## Inputs

- branch: `task-94-grouped-pq-block-kernel`
- prepared head at packet time: `4c796439db80b160a841fd749f03d26e2072a9ec`
- restore snapshot: `snap-0e9c7743263e61d70`
- cloud profile: use the approved Graviton 4 profile for the run and record
  the exact value in the execution packet manifest
- production ARM target: Graviton 4 / Neoverse V2 / SVE2
- quant: `grouped_pq`
- storage format: IVF `pq_fastscan`; DiskANN forced grouped-PQ prefilter where
  applicable
- local suite base:
  `reviews/task-94/025-local-bench-matrix/artifacts/task94-local-pqfastscan-matrix-suite.json`

## Evidence Owed

The Task 94 Graviton 4 packet must include:

1. `Isa::Sve2` asserted by a grouped-PQ parity test on the Graviton 4 host.
2. Measured runtime vector length reported verbatim from `cntw`; label
   `sve2-128` only if the measurement returns 4 lanes.
3. Real NEON parity execution from
   `grouped_pq_neon_backend_matches_scalar_reference_bits_when_available`,
   asserting `Isa::Neon` on-host.
4. Direct `[block-kernel-counters]` rows with
   `surface=ivf quant=grouped_pq isa=sve2 kernel_*` for whole-block work.
5. Direct scalar-tail rows under `isa=scalar`, not under the dispatched ISA.
6. DiskANN grouped-PQ counter rows where the forced grouped-PQ prefilter surface
   is valid, while avoiding a DiskANN speedup claim unless kernel share is
   material.
7. Per-AM matrix for IVF and DiskANN, with recall equality and p50/p95/p99
   latency deltas.
8. Packet 026 closeout notes carried into the interpretation:
   - IVF PqFastScan batch scoring bypasses suffix-max `min_ip_to_keep`
     pruning, so `posting_pruned_by_bound = 0` is expected on batched postings.
   - The IVF PqFastScan kernel path remains opt-in behind
     `ec_ivf.scratch_soa_batch_decode` unless a later approved packet flips the
     default with evidence.

## Provisioning Plan

Use the snapshot; do not recreate corpus or database state. Build missing
Task 94 PqFastScan/grouped-PQ indexes only if the restored snapshot does not
already contain the exact reloption surface required by the matrix.

Future approved command shape:

```sh
ecaz cloud up \
  --profile <approved-graviton4-profile> \
  --from-snapshot snap-0e9c7743263e61d70 \
  --git-ref task-94-grouped-pq-block-kernel

ecaz cloud install \
  --profile <approved-graviton4-profile> \
  --git-ref task-94-grouped-pq-block-kernel
```

Record the actual profile, instance id, region, host type, PG socket/port, and
installed backend SHA in the execution packet manifest.

## On-Host Unit Evidence

Run on the Graviton 4 host after installing the branch:

```sh
cargo test grouped_pq_neon_backend_matches_scalar_reference_bits_when_available --lib -- --nocapture --color never
cargo test grouped_pq_sve_backend_matches_scalar_reference_bits_when_available --lib -- --nocapture --color never
cargo test grouped_pq --lib -- --nocapture --color never
```

Expected interpretation:

- NEON test must execute the NEON hook and assert `Isa::Neon`.
- SVE test must execute the SVE hook and assert `Isa::Sve2` on Graviton 4.
- SVE vector lanes must be captured from
  `runtime_sve_vector_lanes_for_test()`. Convert the returned lane count to a
  label only after measurement, for example 4 lanes -> `sve2-128`.

If the SVE test reports `Isa::Sve` or no measured lane count on Graviton 4,
stop and file feedback rather than continuing to performance claims.

## Suite Evidence

Use `ecaz bench suite`; do not add shell sweepers. Packet 025's local matrix
suite is the current Task 94 suite shape. For the AWS pass, either run the
same suite shape against restored standard-corpus tables or create a packet-local
Graviton suite JSON that preserves the same axes:

- IVF PqFastScan rerank-off, batch off/on, corpus sizes 10k/50k/100k or the
  approved standard corpus set for Task 99 aggregation.
- `ec_ivf.scratch_soa_batch_decode=true` only for batch-on cells.
- DiskANN grouped-PQ forced prefilter cells only where the restored/indexed
  surface is valid.
- direct block-kernel counter collection enabled for latency cells.

Future approved command shape:

```sh
target/debug/ecaz \
  --database postgres \
  --host <pg-socket-or-host> \
  --port <pg-port> \
  --log-file reviews/task-94/<packet>/artifacts/suite-run-cli-graviton4.log \
  bench suite run \
  --config reviews/task-94/<packet>/artifacts/task94-graviton4-pqfastscan-suite.json \
  --artifact-dir reviews/task-94/<packet>/artifacts \
  --manifest-output reviews/task-94/<packet>/artifacts/suite-manifest-graviton4.json \
  --results-output reviews/task-94/<packet>/artifacts/results-graviton4.jsonl
```

If the packet uses the packet 025 suite config directly, record that exact
config path in the manifest. If it creates a Graviton-specific config, commit
the config in the execution packet before running it.

## Counter Extraction Checklist

The execution packet must quote direct lines like:

```text
[block-kernel-counters] surface=ivf quant=grouped_pq isa=sve2 kernel_candidates=... kernel_elapsed_ms=...
[block-kernel-counters] surface=ivf quant=grouped_pq isa=scalar scalar_candidates=... scalar_elapsed_ms=...
[block-kernel-counters] surface=diskann quant=grouped_pq isa=sve2 kernel_candidates=... kernel_elapsed_ms=...
```

Do not use `[task87-counters]` compatibility lines as the primary evidence.

## Stop Conditions

- No `isa=sve2` kernel rows on Graviton 4: stop and debug dispatch/counters.
- SVE vector length not measured: stop and rerun the unit hook with
  `--nocapture`; do not infer width from host class.
- NEON parity hook early-returns on Graviton 4: stop and debug target/runtime
  detection.
- IVF recall differs between batch-off and batch-on: block and triage.
- DiskANN kernel share remains tiny: report counter attribution without making
  a speedup claim.
- Small-cell IVF latency regresses while `posting_pruned_by_bound = 0`: explain
  as the packet 026 pruning-vs-throughput trade, not as a pure kernel failure.

## Carry-Forward Evidence

- Packet 025 is the approved local Intel/AVX2 benchmark matrix:
  `reviews/task-94/025-local-bench-matrix/`.
- Packet 026 documents the closeout pruning/GUC interpretation:
  `reviews/task-94/026-closeout-doc-notes/`.
