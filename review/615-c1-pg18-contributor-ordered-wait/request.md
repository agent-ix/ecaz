# PG18 Contributor Ordered-Wait Diagnostic

## Summary

Please review commit `6c8fd74badab57bcbde0cedd74118228f0d92c67`, which adds a PG18/test-gated diagnostic env:

`TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_ORDERED_WAIT=1`

When the existing contributor diagnostic is enabled, this new env makes non-emitting workers keep `OrderedAfterVisible` hidden rows for the full contributor drain-poll window instead of dropping them after the short ordered-after-visible poll limit. The default path and existing contributor diagnostic path are unchanged unless the new env is also set.

The planner snapshot now reports this ordered-wait diagnostic blocker explicitly so measurement logs can distinguish the normal contributor diagnostic from this timing probe.

## Result

All three 50k/16d randomized PG18 runs pass serial validation:

- Default: contributor counters remain zero; `candidate_missing_serial_ids=[]`; `candidate_extra_ids=[]`.
- Existing contributor diagnostic: 260 hidden publishes, 252 ordered-after-visible classifications, 248 ordered-after-visible drops, zero handoffs, and four output-limit exits.
- Ordered-wait diagnostic: 12 hidden publishes, four ordered-after-visible classifications, zero ordered-after-visible drops, zero handoffs, and four ordered-after-visible poll-limit exits.

The ordered-wait result keeps correctness, but it confirms that waiting longer on ordered-after-visible rows does not make the contributor rows useful on this fixture. They remain behind the visible owner for the full drain window and never become handoff-ready.

## Interpretation

This narrows the path forward: more drain timing or drop-threshold tuning is unlikely to produce useful contributor work. The next implementation slice should move toward distinct/rank-aware contributor frontier work, or a shared serial-rank coordinator that lets contributors claim work that is not already ordered behind the visible emitter.

## Validation

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-ordered-wait-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-ordered-wait-contributor.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_ORDERED_WAIT=1 --log-output target/pg18-parallel-50k-dim16-ordered-wait-diagnostic.log`

## Artifacts

Raw logs are packet-local under `artifacts/`; see `artifacts/manifest.md` for commands and cited key lines.
