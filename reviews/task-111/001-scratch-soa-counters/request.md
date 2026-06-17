# Task 111 Review Request: Scratch SoA Counter Instrumentation

## Scope

This is the first Phase 1 slice for Task 111. It does not change the IVF on-disk layout.

The code checkpoint is `3dcc69f6599c` (`Task 111: instrument IVF scratch SoA reshaping`).

## Change

- Adds IVF EXPLAIN counters for the current row-posting-to-scratch reshaping path:
  - `Scratch SoA Flushes`
  - `Scratch Payload Bytes Copied`
  - `Scratch Heap TID Bytes Copied`
- Records those counters once for every non-empty `IvfPostingScratchSoa` batch processed by the IVF scan path.
- Keeps existing posting pages, visited/scored postings, approximate-scan time, and candidate counters unchanged.

## Rationale

Task 111 Phase 1 needs evidence that row-shaped posting decode and scratch reshaping are material before changing durable page format. These counters expose the current batch-drain frequency and copied payload/TID bytes in the same EXPLAIN diagnostics already used by IVF scan benchmarking.

## Validation

Packet-local artifact:

- `artifacts/cargo-test-ivf-explain.log`

Result:

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2100 filtered out; finished in 0.00s
```

## Notes For Reviewer

This slice intentionally stops before adding a reloption or format-version gate. The next Task 111 slice should use these counters in a same-host Phase 1 audit packet to decide whether dense frozen posting blocks are worth implementing.
