# Task 50 Review Request: SPIRE Custom Scan Cost Helper Safe Signature

## Summary

This slice removes an unnecessary unsafe signature from the SPIRE custom-scan cost estimate helper.

Code commit: `683c25b7bb21ece095fcd89b8a7bd936fd7205f6`

## What Changed

- Converted `estimate_custom_scan_cost` from `unsafe fn` to a safe helper.
- Kept the backend-local planner cost global reads inside narrow internal unsafe blocks.
- Removed the caller-side unsafe block in `CustomScanPlannerRel::estimate_custom_scan_cost`.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1944` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify the helper no longer pushes an unsafe contract to callers.
- Please check the remaining unsafe reads are still scoped to planner cost globals with local safety comments.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for the old unsafe custom-scan cost helper signature and unsafe call wrapper.
- `make UNSAFE_LEDGER=reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1944` (down from packet 324 `1946`)
- Unsafe ledger rows: `1360`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-custom-scan-cost-helper-signature.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
