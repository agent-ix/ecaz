# Task 175: BatANN B1 — Stack-Mode Relay, Cancellation Enabler, Kill-Check

Status: proposed (2026-07-09). Depends on: Task 174.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — first relaying milestone; produces the proceed/de-scope
verdict that gates B2.

## Why

FR-086/FR-087: the actual baton pass. Stack mode is the simple return path
(nested SQL unwind) and already yields the D7 relay-rate measurement under
hash placement. The shared transport also gains interrupt-sliced awaits +
CancelToken propagation — a coordinator-mode enabler (today's
`remote_transport.rs` block_on is uncancellable), ported from the SPIRE
dispatch pattern, landed as its own slice.

## Goal

2/3-node stack-mode reads equal to coordinator mode; cancellable chains;
depth budget + terminal hybrid resume; recorded B2 gate verdict.

## Scope

- Transport relay wiring for the B0 endpoint over the pooled
  `(conninfo,node_id)` transport (full mesh); session identity + epoch
  fingerprint per hop; `application_name = 'ec_distann_relay'`.
- Cancellation enabler slice (own commit): interrupt-sliced awaits +
  downstream cancel, SPIRE dispatch port (detect-inside, return, then
  raise); benefits coordinator mode too (ADR-086 D10).
- FR-089 depth budget, incomplete-state return, terminal coordinator-mode
  resume; `relay_max_depth=0` equivalence.
- FR-087 materialization fix scoped to batann modes
  (`fetch_remote_payloads` local-hit directory re-resolution).
- Link-failure teardown classification; EXECUTE revoked from PUBLIC on the
  relay endpoint (D11).
- Relay counters (FR-084 list) + `debug_relay_trace_notice`.
- Tests: TC-046 core rows (identity under convergence-dominant termination,
  drain-all-local-first, handoff target, occupancy at held depth, cancel
  drill, depth-exhaustion resume, delta-buffer seam, full-mesh check).

## Required Evidence

TC-046 drills green on the loopback multinode fixture; **B1 kill-check**:
informational stack-vs-coordinator latency + relay-rate rows at 2/3 nodes,
release build, packet-local, with a recorded proceed/de-scope verdict for
B2 (ADR-086 Measurement Requirements).

## Non-Goals

Direct mode/mailbox (176); full fault matrix (177); gate matrix (178).

## Acceptance Criteria

1. FR-086 ACs 1–4,6; FR-087 ACs 1–5; FR-089 ACs 1–4,6 green.
2. Coordinator-mode cancel drill green (shared-path enabler proven).
3. Kill-check verdict recorded against ADR-086 before B2 starts.

## References

- FR-086, FR-087, FR-089; ADR-086 D3/D5/D6/D7/D10/D11
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs` (cancel port)
