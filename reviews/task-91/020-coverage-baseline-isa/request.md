# Task 91 Packet 020: coverage baseline for runtime ISA helper

## Summary

This packet fixes the current Test Quality Coverage blocker locally by adding
the missing `quant/isa.rs` critical-path entry to
`fixtures/quality/coverage-baseline.tsv`.

The failed coverage job completed the tests successfully, then failed only the
baseline completeness gate:

```text
coverage baseline missing critical path: quant/isa.rs
```

`src/quant/isa.rs` was introduced by the Task 92 runtime ISA helper work and is
included by `scripts/check_coverage_baseline_complete.sh` because it enumerates
`src/quant/*.rs` except `traits.rs`.

## Code Under Review

- `0e725de616f10af04fa9c70a0826eab1d461bdab`
  `Add runtime ISA coverage baseline`

## Change

- Adds `quant/isa.rs` to `fixtures/quality/coverage-baseline.tsv`.
- Uses the measured merged coverage summary value from the existing failed
  `test-quality-coverage` artifact:
  - `quant/isa.rs` line coverage: `90.48%`

## Local Validation

- `make coverage-baseline-check`
  - Result: `coverage baseline complete for 43 critical paths`
  - Log: `artifacts/coverage-baseline-check.log`
- `scripts/check_coverage_delta.sh artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv artifacts/changed-files.txt`
  - Result: passed for the workflow-equivalent changed-files set.
  - Relevant line: `coverage ok: quant/isa.rs actual=90.48 baseline=90.48`
  - Log: `artifacts/coverage-delta-changed-files-check.log`

## Local Coverage Note

I attempted the full local coverage lane, but this workstation is missing the
optional `cargo-llvm-cov` tool:

```text
missing optional hardening tool: cargo-llvm-cov
```

That failed local setup attempt is recorded in `artifacts/make-coverage.log`.
No GitHub CI run was triggered for this packet.
