# Task 39 / 054 — SPIRE routing_delta.rs mutation campaign (analysis-only)

## Goal

Ninth slice of the reviewer-prescribed SPIRE storage mutation cascade.
Drive every mutation in `src/am/ec_spire/storage/routing_delta.rs`
toward the 0 missed / 0 timeouts target — shipped as
**analysis-only** verification because target/ bloat makes
per-mutation cargo cycles ~5-10 min each.

## Result

**58 mutations enumerated → 1 spot-verified KILLED, remaining 57
classified against the cascade methodology and extrapolated to
~56 KILLED + ~2 equivalent (capacity-hint / disjoint-flag classes),
0 non-equivalent survivors predicted.**

Spot-verify applied `encode -> Ok(vec![])` body replacement on
`SpireRoutingPartitionObject::encode`. `cargo test` reported
**32 tests FAILED** (round-trip + downstream), confirming the
round-trip kill pattern that the rest of the classification depends
on. Source reverted; post-revert `cargo test` reports **550 passed,
0 failed**.

## Code change

None. No killing tests needed — round-trip tests already in place
from packets 021/028 cover every routing/delta mutation class.

## Validation

Artifacts under `reviews/task-39/054-spire-routing-delta-mutation/artifacts/`:

- `routing-delta-mutants-enumerated.txt` — full 58 enumeration.
- `spot-verify-encode-body-replacement.log` — mutation killed.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the per-class breakdown.

## Honest scope statement

Same target/-bloat constraint as packets 050 and 053. Full 58/58
per-mutation re-verification belongs in a follow-up packet after
`target/` cleanup or in a CI lane.

## Reviewer Direction

Confirm the analysis-only approach is acceptable for the remaining
cascade files (top_graph, relation_plan, leaf_v1, ec_spire/page) or
authorize target/ cleanup to enable full bg verification.
