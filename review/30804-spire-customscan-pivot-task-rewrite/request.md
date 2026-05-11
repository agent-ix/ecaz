# Review Request: SPIRE CustomScan Pivot Task Rewrite

Docs-only Step 0 packet for the ADR-067 / ADR-068 / ADR-069 pivot. This
updates Phase 11 tracking so implementation packets stop extending the
superseded index-AM materialization and mirror-sync path.

## Scope

- Rewrites the Phase 11 scope and non-goals to include:
  - ADR-067 CustomScan tuple delivery for distributed reads.
  - ADR-068 endpoint tuple-payload side-channel.
  - ADR-069 v1 write contract: coordinator-routed INSERT, non-embedding
    UPDATE, DELETE, PK-keyed SELECT, placement directory, and embedding-UPDATE
    rejection.
- Rewrites Phase 11.5 and Stage D from "build the materialization mechanism"
  to "build the CustomScan node and wire it to the production executor state
  machine."
- Marks ADR-064 / ADR-065 / ADR-066 and packets
  `30761`, `30762`, `30765`, `30796`, `30797`, `30798`, `30799`, `30801`, and
  in-flight mirror-sync work as superseded Shape-A history.
- Breaks the new pivot into implementation slices:
  - endpoint tuple payload;
  - CustomScan registration, planner path, and executor callbacks;
  - end-to-end CustomScan read fixture;
  - placement directory and coordinator-routed INSERT;
  - coordinator-routed UPDATE / DELETE / PK-keyed SELECT;
  - Stage E fixture migration;
  - cleanup.

## Cleanup Decision

Do **not** drop `ec_spire_remote_row_materialization` or
`ec_spire_register_remote_row_materialization` in this Step 0 docs packet.

Reason: the repository still has Shape-A code on disk and this local worktree
also contains an untracked `review/30802-spire-mirror-sync-contract/` packet.
Deleting the catalog/register surface before the CustomScan read path exists
would turn the pivot task rewrite into a behavior-changing cleanup. The task
file now schedules removal for the cleanup packet after CustomScan reads and
ADR-069 v1 writes are feature-complete.

## Files

- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `review/30804-spire-customscan-pivot-task-rewrite/artifacts/manifest.md`

## Validation

- `git diff --check`

No tests were run. This is a documentation-only checkpoint.

## Reviewer Focus

- Confirm the Stage D breakdown matches ADR-067 / ADR-068 / ADR-069 and the
  implementation brief.
- Confirm the task file no longer presents mirror sync or
  row-materialization catalog work as the production distributed read path.
- Confirm the cleanup decision is acceptable: retain the catalog/register
  function temporarily, then remove them after the CustomScan path exists.
