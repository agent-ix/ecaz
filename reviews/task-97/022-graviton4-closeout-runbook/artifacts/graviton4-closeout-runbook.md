# Task 97 Graviton 4 Closeout Runbook

This runbook prepares the approved-AWS evidence pass for Task 97. It is not an
execution log. Do not run these commands until the project owner explicitly
approves Graviton 4/AWS testing for Task 97.

## Inputs

- branch: `task-97-tq-qjl-block-kernel`
- prepared head at packet time: `56b8a781ade334164c19c7194d32d4d3ec61a8c7`
- restore snapshot: `snap-0e9c7743263e61d70`
- production ARM target: Graviton 4 / Neoverse V2 / SVE2
- qjl32 production fixture: `dim=1024,bits=4,seed=42`
- existing suite config:
  `reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json`

## Evidence Owed

The Task 97 Graviton 4 packet must include:

1. `Isa::Sve2` asserted by a qjl32 parity test on the Graviton 4 host.
2. Measured runtime vector length reported verbatim from `cntw`; label
   `sve2-128` only if the measurement returns 4 lanes.
3. Real NEON parity execution from
   `qjl32_neon_block32_matches_pre_slice_scorer_tolerance_when_available`,
   asserting `Isa::Neon` on-host.
4. Direct `[block-kernel-counters]` rows with
   `surface=<am> quant=turboquant_qjl isa=sve2 kernel_*` for whole-block work.
5. Direct scalar-tail rows under `isa=scalar`, not under the dispatched ISA.
6. Per-AM local-vs-AWS table for IVF, SPIRE, and HNSW.
7. Explicit note that standard `1536d/4-bit` is structurally no-QJL and is out
   of scope for Task 97 qjl32 evidence.

## Provisioning Plan

Use the snapshot; do not rebuild corpus/index data unless the restored snapshot
does not contain the required surface or the qjl32 fixture must be generated.
Kernel-only changes do not require index rebuilds when the on-disk format is
unchanged.

Future approved command shape:

```sh
ecaz cloud up --from-snapshot snap-0e9c7743263e61d70 --git-ref task-97-tq-qjl-block-kernel
ecaz cloud install --git-ref task-97-tq-qjl-block-kernel
```

Record the actual profile, instance id, region, host type, and installed
backend SHA in the execution packet manifest.

## On-Host Unit Evidence

Run on the Graviton 4 host after installing the branch:

```sh
cargo test qjl32_neon_block32_matches_pre_slice_scorer_tolerance_when_available --lib -- --nocapture --color never
cargo test qjl32_sve_block32_matches_pre_slice_scorer_tolerance_when_available --lib -- --nocapture --color never
cargo test qjl32_ --lib -- --nocapture --color never
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

Use `ecaz bench suite`; do not add shell sweepers.

Future approved command shape:

```sh
target/debug/ecaz \
  --database postgres \
  --host /home/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-97/<packet>/artifacts/suite-kernel-on-cli-graviton4.log \
  bench suite run \
  --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json \
  --artifact-dir reviews/task-97/<packet>/artifacts \
  --only-tag kernel_on \
  --manifest-output reviews/task-97/<packet>/artifacts/suite-kernel-on-manifest-graviton4.json \
  --results-output reviews/task-97/<packet>/artifacts/results-kernel-on-graviton4.jsonl

target/debug/ecaz \
  --database postgres \
  --host /home/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-97/<packet>/artifacts/suite-kernel-off-cli-graviton4.log \
  bench suite run \
  --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json \
  --artifact-dir reviews/task-97/<packet>/artifacts \
  --only-tag kernel_off \
  --manifest-output reviews/task-97/<packet>/artifacts/suite-kernel-off-manifest-graviton4.json \
  --results-output reviews/task-97/<packet>/artifacts/results-kernel-off-graviton4.jsonl
```

If the restored AWS host does not use `/home/peter/.pgrx` or port `28818`,
record the actual PG socket/port in the packet manifest and use the host-local
values.

## Counter Extraction Checklist

The execution packet must quote direct lines like:

```text
[block-kernel-counters] surface=ivf quant=turboquant_qjl isa=sve2 kernel_candidates=... kernel_elapsed_ms=...
[block-kernel-counters] surface=ivf quant=turboquant_qjl isa=scalar scalar_candidates=... scalar_elapsed_ms=...
[block-kernel-counters] surface=spire quant=turboquant_qjl isa=sve2 kernel_candidates=... kernel_elapsed_ms=...
[block-kernel-counters] surface=hnsw quant=turboquant_qjl isa=sve2 kernel_candidates=... kernel_elapsed_ms=...
```

Do not use `[task87-counters]` compatibility lines as the primary evidence.

## Stop Conditions

- No `isa=sve2` kernel rows on Graviton 4: stop and debug dispatch/counters.
- SVE vector length not measured: stop and rerun the unit hook with
  `--nocapture`; do not infer width from host class.
- NEON parity hook early-returns on Graviton 4: stop and debug target/runtime
  detection.
- HNSW end-to-end is below the local no-regression floor after the packet 018
  under-octet bypass: stop and document before pursuing more optimization.

## Carry-Forward Evidence

- Packet 018 is the latest local PG18 benchmark evidence after AVX2 octets and
  HNSW under-octet bypass:
  `reviews/task-97/018-qjl32-octet-batch/`.
- Packet 020 adds the forced NEON parity hook needed by this runbook:
  `reviews/task-97/020-qjl32-neon-forced-parity-hook/`.
- Packet 021 refreshes Task 97 status through packet 020:
  `reviews/task-97/021-status-through-packet-020/`.
