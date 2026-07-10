# Task 174: BatANN B0 — Beam-State Seam, Relay-State Serde, Local Relay Identity

Status: proposed (2026-07-09). Depends on: Task 173 (specs).
Owner: coder (to be assigned). One coder, one branch, based on the distann
lane (`task-165-ec-distann-m3` residency until that lane merges).
Priority: P0 — every later milestone builds on this seam.

## Why

FR-085/FR-086 need the FR-081 loop's beam/visited/hits/budgets as an
explicit, serializable object any node can resume. The beam is append-only
today (`scan.rs`), so `(vec_id, code_dist, expanded)` doubles as the visited
set — B0 freezes that invariant and lands the wire format before any
transport work.

## Goal

Pure refactor with zero behavior change, plus `relay_state.rs` serde and a
single-node relay endpoint that reproduces `collect_distann_hits` exactly.

## Scope

- Extract `DistannBeamState` + `distann_local_drain()` from
  `scan.rs:distann_orchestrated_search`; re-express coordinator mode over
  the shared state (mode dispatch seam in `routine.rs:collect_distann_hits`,
  both read paths). Preserve verbatim: `kth_exact_dist` select_nth_unstable
  reordering, early-exit check position, `debug_fail_hop_round` injection
  ordering.
- Expansion budget authoritative, rounds derived (FR-085).
- `relay_state.rs`: DISTANN_RELAY_STATE_V1 encode/decode, structural
  validation (FR-085-AC-6), version reject.
- FR-084 GUC registration (coordination_mode, relay_max_depth default
  min(H,16), relay_wait_timeout_ms 10000, debug_fail/hold_relay_depth,
  debug_relay_trace_notice) + counter surface stubs.
- Local-only `ec_distann_relay_search` (no transport): single-node relay
  identity.
- Tests: TC-045 (round-trip, append-only-beam guard, structural-bounds
  reject, fingerprint-precedes-use, no-heap_tid/no-conninfo inspection,
  GUC drills, amgettuple dispatch).

## Required Evidence

Existing FR-081 unit/pg tests green over the refactor (FR-084-AC-1);
TC-045 suite green. Per checkpoint rules this is a risky-refactor slice —
run the focused ec_distann test set.

## Non-Goals

Node-to-node transport (175); mailbox (176); benches (178).

## Acceptance Criteria

1. FR-085 ACs 1–6 green; FR-086-AC-5 (single-node relay identity) green.
2. FR-084-AC-1/2 green; coordinator-mode scans report zero relay activity.
3. No change in any existing distann test expectation.

## References

- FR-084, FR-085, FR-086-AC-5; ADR-086 D2/D8
- `plan/design/batann-state-passing-coordination.md` (B0, reuse map)
