# Task 39 coverage doc policy cleanup

## Summary

Responds to reviewer feedback on packets 018 and 023 by removing the live
coverage baseline snapshot from `docs/hardening.md`.

The policy doc now points at `fixtures/quality/coverage-baseline.tsv` as the
versioned baseline source of truth. Per-packet request files and manifests
remain responsible for citing raw coverage summaries when rows are raised.

## Code under review

- Commit: `74e5a5229bb6536d8aeac2f4df3c7d91e05c33f2`
- Changed file: `docs/hardening.md`

## Validation

- `git diff --check HEAD~1..HEAD` passed.

## Notes

- No tests were run. This is documentation cleanup only.
- This intentionally removes the "Baseline sources" list and the "Critical
  area" coverage table instead of updating them.
