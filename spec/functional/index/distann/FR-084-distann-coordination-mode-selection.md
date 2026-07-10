---
id: FR-084
title: Distann Coordination Mode Selection
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-075"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-084: Distann Coordination Mode Selection

## Description

The scan coordination strategy SHALL be selected per query by the session GUC
`ec_distann.coordination_mode` with values `coordinator` (default),
`batann_stack`, and `batann_direct` (ADR-086 D1). All modes execute over the
same index, the same published epoch, and the same FR-081 search semantics;
the mode changes only which node advances the beam and how the result
returns.

## Behavior

- `ec_distann.coordination_mode` SHALL be a Userset enum GUC defaulting to
  `coordinator`; an unrecognized value SHALL be rejected at SET time.
- Supporting GUCs: `ec_distann.relay_max_depth` (integer ≥ 0; default =
  min(effective hop-round budget H, 16) per ADR-086 D6),
  `ec_distann.relay_wait_timeout_ms` (direct mode only; default 10000 ms),
  and the debug/fault-injection GUCs `ec_distann.debug_fail_relay_depth`
  (fail a relay at 0-based depth N), `ec_distann.debug_hold_relay_depth`
  (stall a drain at depth N so drills can observe peak occupancy and land
  mid-drain faults deterministically), and
  `ec_distann.debug_relay_trace_notice` (per-drain NOTICE trace:
  frontier-ownership split, handoff target, head-descent skip) — all
  default-off, NFR-020 posture.
- Mode dispatch SHALL live in the shared scan routine
  (`collect_distann_hits`) so both read paths honor the GUC: the
  `amgettuple` path (eager search at rescan per FR-081, cursor over the
  finished heap) and the multi-node CustomScan path.
- FR-083 delta-buffer results SHALL merge on the coordinator as a
  post-search step in every coordination mode; relay drains never observe
  the delta buffer (same posture as materialization, FR-085).
- On a single-node roster (empty/absent roster), `batann_stack` and
  `batann_direct` SHALL execute the local search path and produce results
  identical to `coordinator` mode; the effective mode SHALL be recorded in
  the scan counters.
- With `relay_max_depth = 0`, both batann modes SHALL degenerate to
  `coordinator` mode behavior (ADR-086 D6).
- The scan SHALL surface the coordination mode and the normative relay
  counter set — `relay_hops`, `relay_depth_max`, `relay_depth_histogram`,
  `state_bytes_max`, `state_bytes_total`, `drains_executed`,
  `head_descents`, `handoffs_per_node`, `fallback_resumed`,
  `relay_journeys` (deepen-on-demand re-runs) — via the FR-081 counter
  surface (EXPLAIN, `ec_distann.scan_profile_notice`, bench pipeline step).
  Derived metrics (e.g. relay-rate-per-hop-round = relay_hops ÷
  drains_executed) SHALL be computed from these fields with the formula
  pre-registered where cited (NFR-022).
- Relay and delivery sessions SHALL set a fixed `application_name` tag
  (`ec_distann_relay`) so drills and operators can identify relay backends
  in `pg_stat_activity` per instance.
- The coordination mode SHALL NOT participate in the epoch fingerprint
  (FR-082): mode changes never invalidate epochs or caches.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-084-AC-1 | Default mode is `coordinator`; with the GUC unset, the full pre-existing FR-081 unit/pg test suite passes unchanged over the refactored loop (B0 exit criterion) | Test |
| FR-084-AC-2 | Invalid mode values are rejected at SET time | Test |
| FR-084-AC-3 | Single-node roster: batann modes return results identical to coordinator mode | Test |
| FR-084-AC-4 | `relay_max_depth = 0`: batann modes return results identical to coordinator mode on a multinode fixture | Test |
| FR-084-AC-5 | Mode and relay counters appear in EXPLAIN / profile notice output in every mode | Inspection |
| FR-084-AC-6 | Mode dispatch covers both the amgettuple and CustomScan read paths | Test |

## Dependencies

- **Upstream**: [FR-081](./FR-081-distann-query-orchestration.md),
  [FR-075](./FR-075-ec-distann-access-method-surface.md); ADR-086 D1/D6
- **Downstream**: [FR-086](./FR-086-distann-relay-endpoint-local-drain.md),
  [FR-087](./FR-087-distann-stack-return.md),
  [FR-088](./FR-088-distann-direct-return.md),
  [NFR-022](../../../non-functional/NFR-022-distann-batann-mode-bench-gate.md)
