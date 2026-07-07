# Task 146 Packet 003: Single-Instance Suite Config

## Request

Please review the Task 146 single-instance suite configuration before
execution.

This packet adds the checked-in `ecaz bench suite` config for Task 146's
single-instance attribution matrix. It is the local counterpart to packet 002's
3-worker multinode config. It does not run the matrix.

## Scope

The suite config covers the packet-001 preregistered shapes:

- scales: 10k, 50k, 100k
- shapes: S1-S6
- runner steps: load, truth-cache recall, storage, spire-pipeline
- nprobe sweep: `8,16,32,64,96`
- queries: 200
- `source_identity=include`
- no closure/ratio pruning
- no bound-prune candidate
- no Task 145 rerank-economy promotion

Step count: `57`.

```text
18 load
3 recall
18 storage
18 spire-pipeline
```

## Evidence

Packet artifacts:

- `artifacts/suite-task146-pareto-single-instance.json`
- `artifacts/dry-run-suite-manifest.json`
- `artifacts/dry-run.log`
- `artifacts/audit.log`

Validation run:

```text
jq empty reviews/task-146/003-single-instance-suite-config/artifacts/suite-task146-pareto-single-instance.json
target/release/ecaz bench suite audit --config reviews/task-146/003-single-instance-suite-config/artifacts/suite-task146-pareto-single-instance.json
target/release/ecaz bench suite run --config reviews/task-146/003-single-instance-suite-config/artifacts/suite-task146-pareto-single-instance.json --dry-run --manifest-output reviews/task-146/003-single-instance-suite-config/artifacts/dry-run-suite-manifest.json --log-file reviews/task-146/003-single-instance-suite-config/artifacts/dry-run.log
```

Key result:

```text
[suite:task146-pareto-single-instance] audit passed: 57 steps
```

The dry-run manifest contains 57 steps with this distribution:

```text
load=18
recall=3
storage=18
spire-pipeline=18
```

## Notes

- The `ecaz corpus load` SQL builder adds `INCLUDE (source_identity)` when
  `source_identity=include` is present, so the single-instance config preserves
  the Task 146 source-identity requirement.
- This config does not close packet 001's HNSW/IVF anchor-table obligation or
  epoch-cache-engagement reporting obligation. Those remain owed by the matrix
  execution/results packet.

## Review Focus

1. Confirm this config matches packet 001's six frozen shapes for the
   single-instance half.
2. Confirm the 57-step suite shape is acceptable for the local attribution run.
3. Confirm whether packet 002 and packet 003 together are sufficient suite
   prereqs to start the Task 146 matrix, or whether anchor-table config should
   be reviewed first.

