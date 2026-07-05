# Review Request: SPIRE Production Consistency Policy Preflight

## Summary

Code checkpoint: `ba10fb410640fb9f2ca20a9fe8c4517b1ea420ff`

This slice closes the reviewer-requested C4/C5 preflight gap for active-epoch
consistency policy mismatch:

- Added `ec_spire_remote_search_production_policy_summary(...)` for explicit
  consistency-mode preflight.
- Added `ec_spire_remote_search_production_policy_session_summary(...)` for the
  C5 session-GUC source-of-truth path.
- The summaries return active/requested policy labels, source attribution,
  status, `failure_category`, `failure_action`, next step, and recommendation.
- A session-degraded request against a strict active epoch now returns
  `consistency_mode_mismatch` / `fail_closed` as a row instead of forcing C5 to
  infer policy failure from the existing fail-fast dispatch planner error.
- The Phase 11 task and production coordinator design docs now include the
  broader remaining production-readiness ladder and state that future
  query-level options may override the GUC for one statement, but must not
  replace the GUC contract.

The existing dispatch-planning surfaces remain fail-fast on mismatch. This
preflight is the intended C5 guard before those surfaces are used.

## Key Files

- `src/lib.rs`
- `src/am/mod.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 prod_consistency_policy_summary_mode_mismatch`
- `git diff --check -- src/am/mod.rs src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is a separate row-returning policy preflight the right C5 contract while
  preserving the existing dispatch-planning fail-fast behavior?
- Are the shortened SQL function names clear enough:
  `ec_spire_remote_search_production_policy_summary` and
  `ec_spire_remote_search_production_policy_session_summary`?
- Does the production-readiness ladder capture the right next ordering before
  AM integration and Stage D remote heap finalization?
