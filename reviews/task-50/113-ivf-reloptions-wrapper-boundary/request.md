# Task 50 Review Request: IVF Reloptions Wrapper Boundary

## Summary

This packet reviews commit
`3b0d85b3af6857f660a7772ab2b4fffde860f344`, which removes caller-side unsafe
blocks from IVF string reloption reads.

The slice removes `3` direct unsafe blocks from `src/` (`1675 -> 1672`).

## What Changed

- Made IVF `read_string_reloption` safe to call, with raw reloption offset and
  C-string reads retained inside that helper.
- Removed repeated caller-side unsafe wrappers for `storage_format`,
  `quantizer`, and `rerank` reloption reads.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P7 Reloptions And C String Contracts: IVF string reloption decoding now has a
  safe helper boundary with local residual ownership for raw C string reads.
- IVF/RaBitQ remains one of the priority production surfaces; this packet
  continues the IVF cleanup after prior SPIRE slices.

## Evidence

- Code diff: `artifacts/code-diff.patch`
- Validation: `artifacts/cargo-check-pg18-bench.log`
- Whitespace check: `artifacts/git-diff-check.log`
- Unsafe count: `artifacts/src-unsafe-block-count-after.log`
- Count summary: `artifacts/count-summary.md`
- Ledger: `artifacts/unsafe-ledger-after.jsonl`
- Ledger generation/check logs:
  `artifacts/unsafe-ledger-generate.log`,
  `artifacts/unsafe-ledger-check.log`

## Result

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1675 | 1672 | -3 |
| `src/am/ec_ivf/options.rs` | 7 | 4 | -3 |
| `src/` unsafe ledger rows | 1675 | 1672 | -3 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 3b0d85b3^ 3b0d85b3`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1672` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
