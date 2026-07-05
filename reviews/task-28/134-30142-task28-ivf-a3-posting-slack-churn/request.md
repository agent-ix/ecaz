# Review Request: Task 28 IVF A3 Posting-Slack Churn

## Summary

This packet records the A3 rotating-window churn closure attempt after commit
`419a0713`.

Commit `419a0713` adds `posting_slack_percent`, a build-time IVF reloption
that reserves empty posting pages inside each non-empty list's original
`head_block..tail_block` range. The intent is to let vacuum/refill churn reuse
list-local pages before the live insert path falls through to `P_NEW`, where
rows from different lists interleave and widen every affected list range.

The default remains `0`; this packet measures the explicit churn-oriented
setting `posting_slack_percent = 50`.

## Result

Acceptance shape: 100k live rows, rotating 25k-row delete window, `VACUUM
(ANALYZE)`, refill back to 100k rows, 10 cycles, `nlists in {32,64}`.

The slack-50 run stayed flat for both list counts:

| nlists | cycle 1 index bytes | cycle 10 index bytes | growth |
|---:|---:|---:|---:|
| 32 | 13,623,296 | 13,623,296 | 0.0% |
| 64 | 13,901,824 | 13,901,824 | 0.0% |

Final page-ownership diagnostics show the flat size is not hiding deleted
posting tombstones or cross-list posting pages:

| nlists | posting blocks | posting tuples | heap TID refs | deleted postings | cross-list blocks | mixed blocks |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 1057 | 100000 | 100000 | 0 | 0 | 0 |
| 64 | 1096 | 100000 | 100000 | 0 | 0 | 0 |

## Interpretation

This closes the concrete rotating-window failure reported in packet 30141 for a
churn-oriented build setting. Without slack, the same workload grew from
9,043,968 to 11,198,464 bytes at n32 and from 9,175,040 to 11,829,248 bytes at
n64, with extensive cross-list page ownership. With list-local slack, the index
starts larger but stays flat through the 10-cycle 25% churn run.

This is not an infinite-churn shrink claim. It is an explicit tradeoff: reserve
space at build time to keep later delete/refill churn inside each list's range.
If a workload exhausts the reserve, the next structural fix is list extent/free
space metadata.

## Validation

- `cargo test -p ecaz --lib am::ec_ivf::build::tests`
- `cargo test -p ecaz --lib storage::page::tests::data_page_chain_appends_empty_pages_contiguously`
- `cargo test -p ecaz-cli ivf_vacuum_scale`
- `cargo pgrx test pg18 test_ec_ivf_posting_slack_reloption_reserves_list_range`
- `git diff --check`

## Artifacts

- `artifacts/ivf_a3_100k_rotating_slack50.log`
- `artifacts/page_ownership_slack50.sql`
- `artifacts/page_ownership_slack50.log`
- `artifacts/manifest.md`
