# Review Request: SPIRE Phase 13e AWS Leaf Materialization Gate

Task: Task 30 Phase 13e
Code commit: `ba5ec52afb5f5ddfe1d550ae1f73fb702d0677e1`

## Summary

This checkpoint adds a fail-closed AWS registration guard after static remote
placement publication.

For each remote in `distributed-placement-plan.json`, `scripts/spire-aws/register.sh`
now:

- extracts the coordinator-assigned available leaf entries for that `node_id`
  as `leaf_pid` plus `effective_assignment_count`,
- extracts the available leaf entries from the remote's registered index,
- sorts both packet-local files, and
- fails if any coordinator-assigned leaf PID/count entry is missing on the
  remote.

The failure message explicitly states that leaf-owned materialization is
required before distributed SPIRE reads are valid. This prevents row-hash shard
loads from being misrepresented as production distributed SPIRE placement when
the coordinator routes by leaf PID.

## Validation

Artifacts are under
`reviews/task-30/966-spire-phase13e-aws-leaf-materialization-gate/artifacts/`.

- `bash -n scripts/spire-aws/register.sh` passed.
- `git diff --check HEAD` passed.
- `shellcheck` is not installed in this environment; the packet records that
  availability check.

## Scope Notes

This is a correctness gate, not the materialization primitive itself. It makes
the current AWS path fail safely until remotes contain coordinator-owned leaf
objects/rows with matching leaf PIDs and assignment counts.
