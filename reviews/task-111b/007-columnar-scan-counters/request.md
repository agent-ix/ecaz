# Task 111b Review Request: Columnar Scan Counters

- Code commit: `643928e947bb3dfb42b8074427bd3052c5e0179a`
- Packet: `reviews/task-111b/007-columnar-scan-counters`
- Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`
- Follow-up to: `reviews/task-111b/004-columnar-scan-vacuum/feedback/2026-06-17-01-reviewer.md`

## Summary

This checkpoint adds the forward-looking counters requested in packet-004 feedback before the 111b benchmark slice.

Columnar frozen-list scans now report:

- `stats_columnar_frozen_lists_visited`;
- `stats_columnar_postings_visited`;
- `stats_columnar_logical_bytes_copied`.

The debug gettuple counter snapshot exposes matching fields, EXPLAIN property rendering includes them, and the columnar PG fixture now asserts that columnar postings no longer charge to `dense_postings_visited`. Dense coalesced flush counters still describe the copy into scorer scratch for the 111b copy-based scan.

These counters give 111b a measurable copy baseline and let 111c prove the score-in-place path drives the columnar logical copy bytes toward zero.

## Validation

See `artifacts/manifest.md`.

- `cargo test -q ivf_explain --lib`
  - `2 passed; 0 failed; 0 ignored; 0 measured; 2125 filtered out`
- `cargo test -q test_ec_ivf_columnar_frozen_lists_scan_insert_vacuum --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`
