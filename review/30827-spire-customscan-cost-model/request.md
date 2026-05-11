# Review Request: SPIRE CustomScan Cost Model

## Scope

This packet replaces the provisional `EcSpireDistributedScan` CustomPath cost
from packet `30809`.

The old planner path used `startup_cost = 0` and `total_cost = LIMIT rows` so
the CustomScan path would be selected while executor wiring landed. Now that the
read path is end-to-end, this slice adds a production-shaped cost model with
explicit terms for the work the CustomScan path performs.

This slice:

- Carries the selected index eligibility row into `add_custom_scan_path(...)`.
- Replaces `startup=0,total=LIMIT` with a calibrated model that charges:
  - coordinator routing traversal by bounded remote placement count;
  - per-remote dispatch startup by available remote fanout;
  - bounded heap-rerank work by `LIMIT * fanout`, capped by base rel rows;
  - merge CPU by output rows and fanout;
  - tuple delivery by output rows.
- Keeps the path cheap enough for eligible remote-placement
  `ORDER BY ... LIMIT` queries to continue selecting
  `Custom Scan (EcSpireDistributedScan)`.
- Updates `ec_spire_custom_scan_status().next_step` to the remaining
  ADR-069 write path work.
- Updates the Phase 11 tracker to mark the provisional cost model replaced.

This is still a calibrated planner model, not a benchmark-derived latency
model. It closes the structural gap called out in the task tracker: the path now
accounts for routing, fanout, heap/merge, and tuple delivery instead of using a
placeholder LIMIT-only cost.

## Validation

- `cargo test custom_scan_cost --lib`
  - Passed: 2 tests.
- `cargo test customscan_explain_remote_order_limit --lib`
  - Passed: 1 PG18 test and still selects
    `Custom Scan (EcSpireDistributedScan)`.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Confirm the cost terms cover the intended production work categories without
  over-penalizing the remote CustomScan path on small `LIMIT` queries.
- Confirm the fanout/output-row monotonicity tests are the right focused unit
  coverage for this planner model.
- Confirm updating `next_step` to only `add ADR-069 write path` is accurate.

## Artifacts

- `review/30827-spire-customscan-cost-model/artifacts/manifest.md`
- `review/30827-spire-customscan-cost-model/artifacts/cargo-test-custom-scan-cost-lib.log`
- `review/30827-spire-customscan-cost-model/artifacts/cargo-test-customscan-explain-lib.log`
- `review/30827-spire-customscan-cost-model/artifacts/cargo-fmt-check.log`
- `review/30827-spire-customscan-cost-model/artifacts/git-diff-check.log`
- `review/30827-spire-customscan-cost-model/artifacts/git-diff-cached-check.log`
