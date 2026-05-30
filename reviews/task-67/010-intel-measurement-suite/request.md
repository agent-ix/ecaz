# Task 67 Review Request: Intel Measurement Suite

## Scope

This packet adds the checked-in `ecaz bench suite` configuration needed for
Task 67 Slice J Intel validation.

It is a measurement-prep packet, not the final measurement result packet. The
suite has been audited and dry-run locally; it has not been executed because
the remaining acceptance criteria require running on the Intel benchmark host.

Current head:

- `0471c4cfe2f738ee7353cc5bd99a01bef289d6e1`

## Suite Coverage

Suite config:

- `artifacts/task67-intel-suite.json`

Generated dry-run manifest:

- `artifacts/suite-manifest.json`

The suite has 10 steps:

- restart PG18 with `ECAZ_SIMD=scalar`
- load 100k IVF RaBitQ with `quant_bits=1`
- recall sweep for the scalar lane
- latency sweep for the scalar bits=1 lane
- sidecar rerank for scalar `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- restart PG18 with `ECAZ_SIMD=auto`
- load the matching auto-SIMD 100k IVF RaBitQ fixture
- recall sweep for the auto-SIMD lane
- latency sweep for the auto-SIMD bits=1 lane
- sidecar rerank for auto-SIMD `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`

This covers the Task 67 Slice J measurement surface: pre/post throughput,
recall delta, the four bits=8 variants, and the bits=1 batched path.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.log`.

- `target/debug/ecaz bench suite audit --config ...` passed.
- `target/debug/ecaz bench suite run --dry-run --manifest-output ...` passed and wrote `suite-manifest.json`.
- `target/debug/ecaz bench suite status --manifest ...` reported
  `completed=0 failed=0 skipped=0 dry_run=10 missing_artifacts=0 stale=0`.

## Remaining Task 67 Work

To complete Task 67, run this suite on the Intel benchmark host without
`--dry-run`, generate the suite report/results, and publish the final Slice J
measurement packet with the actual throughput ratios and recall deltas.
