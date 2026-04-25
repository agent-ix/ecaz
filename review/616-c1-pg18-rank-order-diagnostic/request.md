# PG18 Rank-Order Contributor Diagnostic

## Summary

Please review commit `f985259d64c41e2b9565094ec45fe60af8b75d34`, which adds a PG18/test-gated rank-order diagnostic:

`TQVECTOR_PG18_PARALLEL_RANK_ORDER_DIAGNOSTIC=1`

Scalar graph outputs now publish the existing `approx_rank` hint into the shared coordinator. Default coordinator ordering remains score-first. With the rank-order diagnostic env enabled, shared coordinator comparisons use the local serial-rank hint when both rows provide one.

## Result

The diagnostic is intentionally not a production path. It answers whether local per-worker rank hints are enough to make contributor handoffs safe.

- Default lane still passes serial validation.
- Existing contributor diagnostic still passes serial validation and remains behaviorally unchanged: 260 hidden publishes, zero handoffs, 248 ordered-after-visible drops.
- Contributor plus rank-order diagnostic creates one foreign-selected handoff, but serial validation fails. The candidate output swaps IDs `1777` and `35325` relative to serial output while still reporting `candidate_missing_serial_ids=[]` and `candidate_extra_ids=[]`.

That is a useful negative result: local rank hints can create contributor handoffs, but they are not a safe serial-order source. The next path needs a shared/global serial-rank coordinator, not a local rank comparator.

## Validation

Source validation passed:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `git diff --check`

Measurement commands:

- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-rank-order-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-rank-order-contributor.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --env TQVECTOR_PG18_PARALLEL_RANK_ORDER_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-rank-order-diagnostic.log`

The third command is expected to return nonzero for this packet because the diagnostic proves local rank ordering is unsafe.

## Artifacts

Raw logs are packet-local under `artifacts/`; see `artifacts/manifest.md` for commands and cited key lines.
