# Task 92 Packet 010: Bench Suite Quant Axis Markers

## Summary

This packet adds the first Phase 5 bench-suite foundation for the all-quant,
all-index rollout:

- suite steps can now carry quant and ISA axis metadata through tags such as
  `quant=turboquant`, `quant=rabitq`, and `isa=sve2`;
- suite steps can declare `kernel_status=valid`, `missing_kernel`,
  `structurally_absent`, or `invalid_config`;
- non-valid kernel cells are selected but non-runnable, recorded as skipped in
  `suite-manifest.json`, and emitted as `kernel_cell` rows in results
  extraction;
- result rows now include `quant`, `isa`, and `kernel_status` context when the
  step provides those fields.

This is deliberately manifest/results plumbing rather than a full benchmark
matrix. It gives the Task 92 rollout a durable way to represent populated cells
and explicit missing-kernel cells before the full all-quant/all-index benchmark
suite is assembled. Graviton 4 work should use `isa=sve2` for the Arm target
cells; this packet does not introduce any Graviton 3 assumptions.

## Code

- `1fab6ef1c981` - `Add bench suite quant kernel markers`

## Validation

Artifacts are packet-local under `artifacts/`:

- `artifacts/cargo-test-bench-suite.log`
  - command: `cargo test -p ecaz-cli commands::bench::suite::tests --no-default-features`
  - result: 43 passed; 0 failed
- `artifacts/git-diff-check.log`
  - command: `git diff --check`
  - result: passed

## Review Notes

The new test coverage exercises both a runnable populated LUT-style cell and a
missing Graviton 4/SVE2-targeted cell:

- `quant_axis_tags_flow_into_manifest_and_missing_kernel_marker`
- `quant_axis_rejects_unknown_kernel_status_marker`

Remaining Task 92 Phase 5 work still needs a checked-in suite config and an
end-to-end dry-run/sample result packet once the broader quant/index matrix is
ready.
