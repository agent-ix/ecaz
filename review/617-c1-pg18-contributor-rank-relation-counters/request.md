# PG18 Contributor Rank-Relation Counters

## Summary

Please review commits `9f068522a5bb84e101f6145d6c620b68b1e406ef` and `d484e6fa44a672538e62c1fbdf3195803e341987`, which add PG18 contributor publish rank-relation diagnostics without changing the default coordinator ordering.

The new EXPLAIN counters classify a hidden contributor publish against the current visible owner when both are present:

- `Parallel Contributor Publish Rank: Before Visible`
- `Parallel Contributor Publish Rank: Equal Visible`
- `Parallel Contributor Publish Rank: After Visible`
- `Parallel Contributor Publish Rank: Missing Visible`

These counters are folded through the shared DSM contributor counter path, so the elected visible emitter reports totals for all contributor workers. The follow-up commit coalesces the normal hidden-drain classification and rank-relation diagnostic into one shared coordinator observation, so publishing a hidden contributor row does not perform an extra shared-state read just for the rank counters.

## Result

The 50k/16d randomized PG18 contributor diagnostic still passes serial validation and still produces zero handoffs. The new rank counters show why the local-rank path from packet 616 is not a production ordering source:

- 260 hidden publishes.
- 252 publish classifications remain `Ordered After Visible`.
- Rank relation for those publishes is 252 after, 4 equal, 0 before, 0 missing.
- Handoffs remain zero and serial validation passes.

That means local rank hints are present, but normal contributor publishes do not expose a rank-before visible-owner opportunity on this fixture. Packet 616 already showed that using local ranks as the coordinator ordering source can produce an incorrect order, so this packet keeps the rank data diagnostic-only.

## Validation

Source validation passed:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `git diff --check`

Measurement command:

- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-rank-relation-contributor.log`

## Artifacts

Raw log is packet-local under `artifacts/`; see `artifacts/manifest.md` for command metadata and cited key lines.
