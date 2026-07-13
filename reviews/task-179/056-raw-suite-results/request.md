# Review request: structured raw suite results

## Scope

Please review commit `0474ef90983de8acfc64022e3d548ec0bcae7062`,
which adds a narrow generic result surface for `ecaz bench suite` raw steps.

This is the prerequisite runner checkpoint for Task 179's build-gate DML
hot-path A/B. No measurement in this packet depends on unreviewed result
parsing; the measurement will be submitted in the next packet.

## Contract

An artifact declared by a successful raw step may emit:

```text
[suite-result] <metric> key=value key=value ...
```

The suite runner:

- accepts metric names containing only ASCII alphanumeric characters, `_`, or
  `-`;
- requires at least one whitespace-delimited key/value field;
- ignores unrelated output and malformed result markers;
- adds the normal suite/step/connection/tag provenance;
- writes normalized rows to `results.jsonl`; and
- makes the rows available to the existing threshold evaluator.

The runner remains command-agnostic: it does not interpret SQL, benchmark
semantics, or field values beyond the existing raw key/value framing.

## Reviewer focus

1. Confirm malformed or ambient command output cannot become a result row.
2. Confirm raw rows pass through the same provenance and threshold path as
   every other structured suite result.
3. Confirm the prefix is narrow enough not to reinterpret existing raw-step
   logs accidentally.

## Validation

- focused parser test: 1 passed, 0 failed;
- `cargo check -p ecaz-cli`: pass with the pre-existing unrelated unused
  `LoadedDistributedPlacementConfig.path` warning.

Complete exact-SHA logs are packet-local under `artifacts/`.
