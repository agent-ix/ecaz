# Task 30 Phase 12 Handoff

Continue ecaz Task 30 on branch `task-30-spire` from the pushed Phase 11
closeout state. Current pre-handoff HEAD when this file was written:
`dba1a9a5` (`Close Task 30 Phase 11 via packet 30910`).

## Current State

- Phase 9 local SPIRE graph architecture is complete.
- Phase 10 local SPIRE execution/performance architecture is complete.
- Phase 11 is closed by reviewer packet `30910`.
- Functional distributed SPIRE exists:
  - vector `ORDER BY ... LIMIT` reads can return remote rows through
    `EcSpireDistributedScan`;
  - ADR-068 tuple payloads are wired for CustomScan delivery;
  - ADR-069 coordinator-routed INSERT, non-embedding UPDATE, DELETE, PK SELECT,
    and embedding-UPDATE rejection are live;
  - the ADR-064/065/066 materialization catalog/register/mirror-sync path is
    retired;
  - Stage E fault matrix (11 cases) and lifecycle matrix (6 cases) pass
    against the CustomScan build in packet `30895`.
- Phase 12 is the active work: production hardening, performance, local
  readiness, and runbooks before AWS.
- Phase 13 is AWS/RDS-class verification and is blocked on Phase 12 exit.

## Current Worktree Caveat

At handoff time, the worktree had unrelated local WIP:

- `scripts/tests/test_pg17_scratch_psql_socket_resolution.py`
- `scripts/tests/test_resolve_scratch_socket_dir.py`
- untracked `review/30802-spire-mirror-sync-contract/`

Do not revert or sweep those into commits unless the user explicitly scopes
them. This handoff intentionally replaces stale `handoff.md` content.

## Read First

1. `AGENTS.md`
2. `review/README.md`
3. `plan/tasks/task30-phase12-spire-production-hardening.md`
4. `plan/tasks/task30-phase13-spire-aws-verification.md`
5. `review/30896-spire-customscan-architecture-review/request.md`
6. `review/30909-spire-hardening-task-split/feedback/2026-05-12-001-reviewer.md`
7. `review/30910-spire-phase11-closeout/request.md`

Use `plan/tasks/task30-phase11-spire-distributed-production-parity.md` only as
historical context. Its header says not to pick up new work from that file.

## Reviewer Findings To Carry Forward

Reviewer packet `30896` is the durable architecture review. It split remaining
work into:

- H1-H12 hardening:
  concurrent INSERT race, concurrent DELETE collision, 2PC crash recovery,
  cancel/timeout mid-INSERT-prepare, EvalPlanQual/isolation, schema drift,
  trigger JSON type round-trips, predicate edge rejection,
  `max_prepared_transactions`, DDL/write ordering, multi-row INSERT, and
  placement-table contention.
- P1-P9 performance:
  typed tuple transport, placement planner indexed lookup, JSON decode
  allocation retirement, relation-context caching, 2PC latency tradeoffs,
  cost-model calibration, `custom_private` layout cleanup, PK-byte allocation
  cleanup, and async INSERT dispatch.

Reviewer packet `30909` accepted the Phase 12/13 split and added small
planning observations:

- O1: Phase 11 closeout has now reconciled/moved remaining work via packet
  `30910`.
- O3/O4/O7: items that say "evaluate", "decide", or "migration window" need
  measurable exit criteria before they are considered complete.
- O5: Phase 13 deferrals require reviewer acceptance.
- O6: ADR-069 deferred items remain later ADR scope, not Phase 12 unless
  explicitly reopened.
- O8: AWS datasets can be pinned at Phase 13 packet time.

## Phase 12 Task Map

- Phase 12.1: tracker and operator-compatibility reconciliation.
- Phase 12.2: typed tuple transport and JSON retirement.
- Phase 12.3: planner, metadata, and cost hardening.
- Phase 12.4: coordinator-routed write and 2PC hardening.
- Phase 12.5: schema drift, DDL, and type round-trip hardening.
- Phase 12.6: isolation, EvalPlanQual, and negative DML coverage.
- Phase 12.7: multi-instance placement, epoch, and replica readiness.
- Phase 12.8: local multi-store and multi-NVMe readiness.
- Phase 12.9: local production harness and runbook.

AWS work belongs to Phase 13, not Phase 12.

## Suggested First Coding Slices

Follow review-packet discipline: one narrow code/docs slice, one matching
review packet, commit and push each separately.

1. **Phase 12.1 operator-compat cleanup**
   - Add the 0.1.1 -> 0.1.2 migration comment explaining the
     materialization-table create/drop history.
   - Document `requires_remote_row_materialization` ->
     `requires_custom_scan_tuple_delivery`.
   - Document dropped mirror-sync/materialization operator-entrypoint rows.
   - Cross-link packet `30895` to `30770`, `30772`, and `30773`.
   - Record the future 0.2.x cleanup for zero-valued `row_materialization_*`
     shim columns.

2. **Phase 12.3 placement planner gate indexed lookup**
   - Replace the `ec_spire_placement` planner eligibility seqscan with an
     indexed lookup.
   - Add focused coverage proving eligibility remains bounded as placement
     rows grow.
   - This is a small high-impact hardening/perf slice.

3. **Phase 12.2 typed tuple transport design**
   - Start with a design/review packet before large executor changes.
   - Pin the protocol choice: binary composite/record vs per-attribute
     `typsend`/`typreceive`.
   - Define negotiation, JSON fallback window, and removal criteria so the JSON
     bridge does not linger indefinitely.

If choosing between them, start with slice 1 if the next agent is doing docs
cleanup; start with slice 2 if the next agent is ready to code.

## Validation Expectations

- Do not run broad tests by default.
- For docs-only task updates, `git diff --check` is sufficient.
- For planner/executor changes, prefer narrow PG18-focused tests or existing
  multicluster scripts.
- For pgrx-facing behavior, use `cargo pgrx test pg18` only when static review
  is not enough.
- Do not run PG17 unless explicitly asked.
- Do not skip hooks or use `--no-verify`.

## Workflow Guardrails

- Start every turn by scanning `review/` for new feedback.
- Process owned actionable feedback before new work.
- Do not re-triage closed review topics unless a reviewer reopens them.
- Do not make AWS/product-scale claims from local evidence.
- Do not open Phase 13 work until Phase 12 exit criteria are complete or every
  remaining Phase 12 item has accepted reviewer deferral.
- Do not revert unrelated WIP.
- Stage feedback files and review artifacts by exact paths, never broad
  `git add review/`.
- Prefer the `ecaz` operator CLI for local PG18/pgrx workflows when the surface
  exists; otherwise keep scripts packet-local and reproducible.
