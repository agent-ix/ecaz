# Task 146 Packet 002: Multinode Suite Config

## Request

Please review the Task 146 multinode suite configuration before execution.

This packet adds a checked-in `ecaz bench suite` config for the packet-001
preregistered multinode matrix. It does not run the matrix.

## Scope

The suite config covers:

- scales: 10k, 50k, 100k
- shapes: S1-S6 from packet 001
- runner: `spire-local-multinode`
- build/install path: release by default, no `debug_install`
- nprobe sweep: `8,16,32,64,96`
- queries: 200
- projection: `id,source`
- common exclusions: no closure/ratio pruning, no bound-prune candidate, no
  Task 145 rerank-economy promotion

Shape count: `18` steps, matching `3 scales * 6 shapes`.

## Evidence

Packet artifacts:

- `artifacts/suite-task146-pareto-multinode.json`
- `artifacts/dry-run-suite-manifest.json`
- `artifacts/dry-run.log`
- `artifacts/audit.log`

Validation run:

```text
jq empty reviews/task-146/002-multinode-suite-config/artifacts/suite-task146-pareto-multinode.json
target/release/ecaz bench suite audit --config reviews/task-146/002-multinode-suite-config/artifacts/suite-task146-pareto-multinode.json
target/release/ecaz bench suite run --config reviews/task-146/002-multinode-suite-config/artifacts/suite-task146-pareto-multinode.json --dry-run --manifest-output reviews/task-146/002-multinode-suite-config/artifacts/dry-run-suite-manifest.json --log-file reviews/task-146/002-multinode-suite-config/artifacts/dry-run.log
```

Key result:

```text
[suite:task146-pareto-multinode] audit passed: 18 steps
```

The dry-run manifest contains 18 steps.

## Packet 001 Feedback Carried Forward

Packet 001 feedback approved the shape preregistration and added matrix-run
obligations. This config packet does not satisfy those obligations by itself;
the execution/results packet must include them:

- report release HNSW anchors alongside IVF anchors
- report matched-scale 10k/50k/100k IVF and HNSW anchors, not only the 100k IVF
  gate anchor
- include live evidence that Task 142's epoch-cache floor removal is engaged in
  these runs, or identify the exact result/manifest field that proves it
- label the 15% scan gate as a viability band, not Pareto dominance; the
  frontier table must still show whether IVF/HNSW dominate SPIRE at each point
- justify using the permissive 15% end of the spec's 10-15% scan range

## Review Focus

1. Confirm the config matches packet 001's six preregistered shapes and does
   not smuggle in rejected Task 144/145 levers.
2. Confirm the suite shape is acceptable for the multinode half of Task 146's
   matrix.
3. Confirm whether this config should run as-is after packet-001/002 review, or
   whether single-instance config and anchor-table config should be added first
   and reviewed together.
