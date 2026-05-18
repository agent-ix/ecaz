# Review Request: SPIRE Remote Fanout Diagnostic SQL

- Code commit: `f7340890` (`Expose SPIRE remote search fanout plan`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint exposes the remote-search fanout planner through SQL so the
next libpq executor slice has an inspectable request contract:

- adds `SpireRemoteSearchFanoutPlanRow`;
- adds `remote_search_fanout_plan_rows`;
- exports `ec_spire_remote_search_fanout_plan(index_oid, requested_epoch,
  selected_pids, consistency_mode)`;
- validates positive requested epoch, selected PID sign, active epoch match,
  and active consistency-mode match;
- returns one row per local target PID, remote target PID, or degraded skipped
  placement;
- reports `target_kind`, `node_id`, `pid`, and `placement_state`;
- adds PG18 coverage for the local-node fanout rows on a normal two-leaf index;
- updates the Phase 7 task note to record the SQL diagnostic surface.

This remains a diagnostic/planner surface only. It does not open libpq
connections or execute remote searches.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check the SQL output contract: `requested_epoch`, `target_kind`, `node_id`,
   `pid`, and `placement_state`.
2. Check that the SQL prelude mirrors the remote-search endpoint fail-closed
   checks for active epoch and consistency mode.
3. Check that local-only fanout remains visible as `target_kind = 'local'`
   rather than hidden from diagnostics.
4. Check that this diagnostic does not accidentally imply remote transport has
   landed.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search_fanout --no-default-features --features pg18`
  - Result: passed; 4 tests passed, including the SQL diagnostic test.
- `git diff --check`
