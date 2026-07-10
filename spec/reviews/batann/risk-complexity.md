---
id: SR-012
title: Risk-Complexity Analysis of the ec_distann BatANN Spec Batch
type: SpecReview
analysis: risk-complexity
scope: "spec/functional/index/distann/FR-084..089, spec/non-functional/NFR-021..022, spec/adr/ADR-086, spec/tests.md TC-045..048, plan/design/batann-state-passing-coordination.md"
review_set: all
---

## Summary

Technical-risk and volatility scoring of the 10-artifact BatANN
state-passing batch (ADR-086, FR-084..FR-089, NFR-021..NFR-022,
TC-045..TC-048, milestone doc B0–B4), grounded in the implementation the
batch extends (`src/am/ec_distann/{scan.rs, remote_transport.rs,
remote_endpoint.rs, custom_scan.rs, routine.rs, options.rs}`).

The batch is honest about its two structural bets — the D4 send-and-abandon
spike and the D7 hash-placement relay-rate caveat — and its degenerate
anchors (`relay_max_depth = 0` ≡ coordinator, single-node relay identity)
are well chosen. Verified-sound claims: the FR-085 beam-as-visited-set
claim holds against the actual FR-081 loop (`scan.rs:109-214` — the beam
`Vec<(f32, u64)>` never removes entries, `enqueued` == beam vec_ids by the
insertion gate, `expanded ⊆ enqueued`, so `(vec_id, expanded)` over beam
entries reproduces both sets); D10 has direct in-repo precedent in the
SPIRE dispatch layer (cancel-signal select + `CancelToken` propagation +
statement-timeout detection, `ec_spire/coordinator/remote_candidates/dispatch.rs:2418-2556`),
so interrupt-sliced awaits are a port, not an invention; and the D3
no-deadlock argument (A→B→A lands in a fresh backend) is consistent with
the per-backend thread-local pool keyed `(conninfo, node_id)`
(`remote_transport.rs:78-100,151-160`).

The three highest risks: **(1)** the D4 flush guarantee is very likely
unobtainable on stock tokio-postgres (query futures are lazy — nothing is
transmitted until polled, and there is no flush-completion observable on a
current-thread runtime that stops being driven when `block_on` returns), so
direct-lite should be treated as the *probable* shipped form — and FR-088's
headline SHALL ("intermediate nodes SHALL NOT hold the relay chain")
contradicts that fallback, leaving a normative statement that would be
violated by the recorded contingency; **(2)** the D6 default
`relay_max_depth = H` binds to `ECDISTANN_DEFAULT_HOP_ROUNDS = 100`
(`mod.rs:97`), making NFR-021's own worst-case bound 101 backends for a
*single* stack-mode query — beyond stock `max_connections`; **(3)** the D7
kill-check lands only at B4, after the B2/B3 mailbox and drill investment,
even though B1 already delivers everything needed for a cheap
stack-vs-coordinator latency + relay-rate checkpoint at 2–3 nodes. Under
hash placement, coordinator mode expands the whole top-BW frontier across
nodes in one grouped parallel RTT per hop round
(`remote_expand_batch`/`join_all`, `remote_transport.rs:136-253`) while
relay mode visits owners serially with per-hop state serialization, so
"BatANN mode is measurably worse than coordinator mode" is a live program
outcome; the specs state this honestly (D7, NFR-022's report-only latency
row, the promote/iterate/shelve verdict), but the evidence arrives late.

**Risk register** (technical risk × volatility; High rows carry a named
mitigation):

| Req | Tech Risk | Volatility | Drivers | Mitigation |
|-----|-----------|------------|---------|------------|
| FR-084 | Low | Low | GUC + dispatch follow existing precedent (`options.rs:191`, `collect_distann_hits` at `routine.rs:286` already covers both read paths) | AC-1 byte-identical default anchor |
| FR-085 | Low–Medium | Medium | Format claim verified against `scan.rs`; size-envelope arithmetic understated at real defaults (FND-009); `hop_rounds_remaining` vs expansion-budget dual accounting (FND-010) | Guard test freezing the append-only-beam invariant; pin envelope + mailbox cap together |
| FR-086 | Medium | Medium | New drain algorithm (Algorithm 2) changes traversal under the same BW×H cap; budget bookkeeping ambiguity across rounds-based loop vs expansion budget | Counter-asserted ACs (AC-2/AC-4); FND-010 accounting pin |
| FR-087 | Medium | Low | Occupancy default hazard (FND-003); cancellation is a hard prerequisite, currently absent from the distann transport; top-k-identity AC is flaky by construction if the fixture budget binds (FND-005) | Port the SPIRE cancel pattern in B1 (FND-012); pin fixture params |
| FR-088 | High | High | Send-and-abandon spike likely fails (FND-001); shmem mailbox is first-of-kind in this codebase (FND-006); timeout semantics unpinned (FND-013); Description SHALL conflicts with the fallback (FND-002) | Timebox the spike before B2 implementation; reword the SHALL contingently; direct-lite as default plan |
| FR-089 | Low–Medium | Low | Clean design; the splice into the rounds-based FR-081 loop needs a defined rounds↔expansions conversion (FND-010) | Depth-0 equivalence anchor (AC-3/FR-084-AC-4) |
| NFR-021 | Medium | Medium | Bound formula is right but the default depth makes the stated worst case exceed stock `max_connections` (FND-003); envelope figure understated (FND-009) | Cap the default independently of H; restate envelope at real defaults |
| NFR-022 | Medium | Low | 0.001 recall-parity bar at matched BW/H may be structurally unreachable for a traversal that differs by construction (FND-005); latency is report-only (honest); D7 evidence row good | Matched-budget-with-headroom framing; B1 early checkpoint (FND-004) |
| ADR-086 | — | Medium | D4 High (spike), D7 Medium-High program bet with late kill-check, D6 Medium (default), D10 Medium-Low (SPIRE precedent, one caveat: never literal `CHECK_FOR_INTERRUPTS` inside `block_on` — FND-007), D1/D2/D3/D5/D8/D9 Low | B1 checkpoint gates B2 (FND-004); spike verdict recorded against the ADR before B3 (already required) |

**Recommended spike ordering / milestone adjustments** (details in
FND-001/002/004/012): (1) run the send-and-abandon flush spike as a
timeboxed pre-B2 item — it is pure transport, independent of the mailbox,
and its verdict changes FR-088's normative wording; (2) add a
stack-vs-coordinator latency + relay-rate informational checkpoint to B1's
exit criteria and make B2 entry conditional on a recorded proceed/de-scope
verdict; (3) move interrupt-sliced awaits + cancel propagation from B3 into
B1 (an uncancellable blocking relay chain is a bench-host hazard and the
TC-046 cancel drill is already listed under B1's test case); (4) cap the
`relay_max_depth` default independently of H until B4 evidence.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | D4 send-and-abandon flush guarantee is very likely unobtainable on stock tokio-postgres: query futures are lazy (nothing reaches the socket until the future is polled and the Connection task is driven), the transport's current-thread runtime stops being driven the moment `block_on` returns (`remote_transport.rs:90-99,171,384`), and tokio-postgres exposes no outbound-flush-complete signal — the only in-band confirmation (pipelining a probe on the same connection) blocks behind the relay statement's execution, defeating the purpose. Treat direct-lite as the probable shipped form and run the spike timeboxed *before* B2 implementation starts, since its verdict changes FR-088's normative text (FND-002), not just its internals. | ADR-086 D4; FR-088; plan/design/batann-state-passing-coordination.md (B2, Hazard "Direct-mode forwarding"); src/am/ec_distann/remote_transport.rs:78-253 |
| FND-002 | high | FR-088's headline SHALL — "intermediate nodes SHALL NOT hold the relay chain" — is violated by its own recorded fallback: under direct-lite each intermediate's synchronous forward ack means its `ec_distann_relay_search` call returns only after the downstream call returns, i.e. stack-mode occupancy with a mailbox return. No AC covers backend-freeing (AC-1..6 all survive direct-lite), so the ACs are fallback-complete, but the Description and NFR-021's direct-mode occupancy posture would need rewording if the spike fails. Reword now: scope the SHALL to the send-and-abandon variant ("when the flush guarantee holds…") and state direct-lite's occupancy explicitly; require the NFR-022 gate packet to record which variant ran, since direct-lite collapses the stack-vs-direct comparison to the return-trip term alone. | FR-088 (Description, Behavior); ADR-086 D4; NFR-021; NFR-022 |
| FND-003 | high | The D6 default `relay_max_depth = H` is dangerous at real defaults: `ECDISTANN_DEFAULT_HOP_ROUNDS = 100` (src/am/ec_distann/mod.rs:97), so NFR-021's own worst-case bound is 101 backends and 100 pooled connections for a single stack-mode query — beyond stock `max_connections = 100` before any concurrency. Early-exit will usually stop far sooner, but the spec's stated sizing guidance (concurrent_queries × (depth+1)) becomes absurd at the default. Pin the default to min(H, small constant, e.g. 8–16) until B4's relay-rate evidence, and state the H=100 interaction in NFR-021's sizing guidance. | ADR-086 D6; FR-084; FR-089; NFR-021; src/am/ec_distann/mod.rs:97 |
| FND-004 | medium | The D7 program-level risk — BatANN mode measurably worse than coordinator mode under hash placement — is stated honestly (D7 reopen trigger, NFR-022 report-only latency, promote/iterate/shelve verdict), but the earliest evidence lands at B4, after B2's mailbox and B3's drill matrix are paid for. Coordinator mode expands the whole top-BW frontier across all owners in one grouped parallel RTT per hop round (`remote_expand_batch` + `join_all`, remote_transport.rs:136-253); relay mode serializes owner visits with per-hop state serialization, so at 2–3 nodes and near-100% relay rate the structural comparison can already be negative. B1 delivers both modes on the fixture: add a cheap informational stack-vs-coordinator latency + relay-rate checkpoint to B1's exit criteria and gate B2 entry on a recorded proceed/de-scope verdict (de-scope = defer direct mode, keep stack for the B4 record). | ADR-086 D7; NFR-022; plan/design/batann-state-passing-coordination.md (B1/B4); src/am/ec_distann/remote_transport.rs:136-253 |
| FND-005 | medium | The result-equivalence bar is structurally strained: eager local drains expand different candidates than coordinator mode under the same BW×H cap (D9 admits traversal divergence), yet FR-087-AC-1 demands *top-k identity* and NFR-022 demands distinct_recall@10 within 0.001 at matched BW/H. Identity holds only when the fixture converges well inside the budget; if the budget binds, relay mode's locally-biased expansions can legitimately produce a different top-k. Pin the fixture parameters (generous H, convergence-dominant termination, counter-asserted `early_exit`) as part of the AC, or soften to matched-budget-with-headroom parity; otherwise TC-046's flagship assertion is flaky by construction. | ADR-086 D9; FR-087-AC-1; FR-088-AC-2; NFR-022; spec/tests.md TC-046 |
| FND-006 | medium | The D4 shmem mailbox is first-of-kind in this codebase: grep confirms zero existing shmem/latch/LwLock usage anywhere in `src/`. pgrx 0.17 does provide the shmem half (`pg_shmem_init!`, `PgLwLock`, fixed-size Copy structures; requires `shared_preload_libraries=ecaz`, which is already the deployment posture — `crates/ecaz-cli/src/commands/dev/test.rs:320,355`), but there is no safe latch wrapper: cross-backend wakeup needs raw `pg_sys::{SetLatch, WaitLatch}` against the waiter's `PGPROC->procLatch` stored in the slot, plus `WL_EXIT_ON_PM_DEATH` and a transaction-abort callback (`register_xact_callback`) for slot cleanup. All standard PostgreSQL patterns, but budget B2 for a fixed-slot array (not a hash), the shmem_request_hook wiring in `_PG_init` (src/lib.rs:74), and pgrx-API-shape discovery time. | ADR-086 D4; FR-088; NFR-021 (fixed mailbox budget); src/lib.rs:74; crates/ecaz-cli/src/commands/dev/test.rs:320 |
| FND-007 | medium | D10 interrupt-sliced cancellation has direct in-repo precedent and one wording trap. Precedent: SPIRE dispatch races the query future against a poll of PostgreSQL's interrupt flags (`postgres_query_cancel_pending` via dlsym of `InterruptPending`/`QueryCancelPending`), propagates downstream with `tokio_postgres::CancelToken::cancel_query`, and classifies statement timeouts via `get_timeout_indicator` (ec_spire/coordinator/remote_candidates/dispatch.rs:2418-2556, tls.rs:350-358) — port this, do not invent. Trap: ADR-086's "`CHECK_FOR_INTERRUPTS` between slices" must not be read literally inside `block_on` — raising an interrupt there longjmps across live async/runtime frames; the safe shape is detect-inside, return out of `block_on`, then raise (the SPIRE pattern). Residual novelty: multi-hop propagation (every intermediate must both observe local interrupts and cancel its downstream) exceeds SPIRE's single-hop pattern, and the distann transport currently has *no* cancel handling at all (plain `block_on` + `join_all`). | ADR-086 D10; FR-087 (cancellation behavior); src/am/ec_spire/coordinator/remote_candidates/dispatch.rs:2418-2556; src/am/ec_distann/remote_transport.rs:171,384 |
| FND-008 | low | The FR-085 beam-as-visited-set claim is verified against the implementation and is sound: in `distann_orchestrated_search` the beam `Vec<(f32, u64)>` only ever appends (sorted in place, never popped), membership is gated by `enqueued.insert`, seeds enter the beam, and `expanded` only ever receives batch members drawn from the beam — so `(vec_id, code_dist, expanded)` over beam entries reproduces `enqueued` and `expanded` exactly (scan.rs:109-214). Residual risk: any future loop optimization that prunes the beam silently breaks the wire invariant; B0 should land a unit test that freezes append-only membership as an explicit invariant, not just round-trip serde. | FR-085 (beam entries as visited set); ADR-086 D2; src/am/ec_distann/scan.rs:109-214; spec/tests.md TC-045 |
| FND-009 | medium | The state-size envelope is understated at real defaults: NFR-021 says "tens of KB", but the spec's own formula (beam ≤ seeds + BW×H×R × 13 B) at BW=4, H=100, R=32 (mod.rs:90,97,50) gives ≈12,800 entries ≈ 166 KB worst case — which also exceeds the design doc's proposed ~64 KB mailbox inline cap precisely when a query has done the most work, turning a hard-working query into an FR-088-AC-6 oversize error. Restate the envelope with the arithmetic evaluated at the shipped defaults, and pin the mailbox cap policy (inline cap vs DSM overflow — currently an open spec-review question) against that number, not against the paper's 4–8 KB scale. | NFR-021; FR-085 (size envelope); FR-088-AC-6; plan/design/batann-state-passing-coordination.md (wire-format sketch, open items); src/am/ec_distann/mod.rs:50,90,97 |
| FND-010 | medium | Budget bookkeeping is double-entried and the conversion is unspecified: FR-085 carries both `hop_rounds_remaining` and the D8 global BW×H expansion budget, but relay drain rounds expand fewer than BW records (locals only), so rounds×BW ≠ expansions; the FR-089 splice resumes "the FR-081 grouped hop loop" — which is rounds-based (`params.hop_rounds`, scan.rs:124) — "with the remaining BW×H budget", leaving open whether the resume gets ceil(budget/BW) rounds, an expansion-count loop bound, or something else. FR-086-AC-4/FR-089-AC-4 assert the cap but not which accounting is authoritative. Pin one authority (recommend: expansion budget is normative, rounds derived) before B0 fixes the `DistannBeamState` fields. | FR-085; FR-086 (Behavior, AC-4); FR-089 (Behavior, AC-4); ADR-086 D8; src/am/ec_distann/scan.rs:124-215 |
| FND-011 | low | B0's `DistannBeamState` refactor risk on the shared FR-081 loop is genuinely low-medium, not high: the loop is ~120 compact lines behind a mocked trait seam with six focused unit tests (scan.rs:254-484), and both read paths converge on `collect_distann_hits` (routine.rs:286), so the regression surface is well-fenced by TC-040/041 multinode-identity tests plus FR-084-AC-1's byte-identical anchor. The subtle bits to preserve verbatim: `kth_exact_dist`'s in-place `select_nth_unstable` reordering of `hits`, the early-exit check position (after batch selection, before expansion), and the `debug_fail_hop_round` injection point ordering. | FR-084-AC-1; plan/design/batann-state-passing-coordination.md (B0, reuse map); src/am/ec_distann/scan.rs:97-252; src/am/ec_distann/routine.rs:286 |
| FND-012 | medium | B3 is the budget outlier of the milestone table: roughly ten drill classes across two modes on a real multi-instance fixture (mid-chain republish, cancel-chain teardown, killed terminal node, mailbox timeout/orphan/abort, busy-until-drained hygiene, pg_stat_activity leak assertions), each requiring fault orchestration the `distann_multicluster` fixture does not yet have — realistically larger than B1 and B2 combined. Two adjustments: (a) move interrupt-sliced awaits + downstream cancel out of B3 into B1 — a stack chain that cannot be cancelled is unusable on the shared bench host the moment B1 lands, and TC-046's cancel drill is already attributed to B1/B3; (b) keep mailbox lifecycle drills co-located with B2 (they test B2's slot machinery) so B3 shrinks to the cross-cutting fault matrix. | plan/design/batann-state-passing-coordination.md (B1–B3); spec/tests.md TC-046/TC-047; NFR-021 |
| FND-013 | low | FR-088 ships with an unresolved normative decision embedded in its Behavior: direct-mode wait-timeout semantics ("error vs one coordinator-mode rerun") are "pinned by spec review", yet FR-088-AC-4 verifies "per the pinned timeout semantics" — an AC that cannot be written until the pin lands. Same for the mailbox inline-cap-vs-DSM question feeding AC-6. Both are flagged volatility, not oversights, but they must be pinned before B2 tasking (they change the endpoint signature and the drill matrix), and the pin should land as an ADR-086 amendment, not an implementation choice. | FR-088 (Behavior, AC-4, AC-6); plan/design/batann-state-passing-coordination.md (open items) |
| FND-014 | low | FR-084 is the batch's low-risk anchor and correctly placed first: enum/int GUC registration has direct precedent (`register_gucs`, options.rs:191-264, including the NFR-020-style `debug_fail_*` pattern the new `debug_fail_relay_depth` copies), mode dispatch has a single natural seam already covering both read paths (`collect_distann_hits`, routine.rs:286 — confirmed used by amgettuple and the CustomScan), and the two degenerate equivalences (single-node roster, depth-0) give every later milestone an always-on correctness anchor. One caution: AC-1's "byte-identical to pre-BatANN builds" is only as strong as the B0 refactor discipline (FND-011). | FR-084; src/am/ec_distann/options.rs:191-264; src/am/ec_distann/routine.rs:286-460 |

## Reconciliation (2026-07-09, post-review spec revision)

- FND-001 **RESOLVED** — ADR-086 D4 reframed: pre-B2 timeboxed spike with
  direct-lite named the probable shipped form; design doc B2 row matches.
- FND-002 **RESOLVED** — FR-088 Description/Behavior scoped per variant
  (send-and-abandon frees backends; direct-lite has stack occupancy);
  NFR-021 states the direct-lite bound; NFR-022 records the variant run.
- FND-003 **RESOLVED** — D6 default pinned to min(H, 16) across ADR-086,
  FR-084, FR-089, the design doc, and NFR-021's sizing guidance (with the
  H=100 interaction warning).
- FND-004 **RESOLVED** — B1 exit gains the informational
  stack-vs-coordinator latency + relay-rate checkpoint with a recorded
  proceed/de-scope verdict gating B2 (ADR-086 Measurement Requirements,
  design doc B1 row).
- FND-005 **RESOLVED** — D9 split into fixture bar (D9a, valid only under
  convergence-dominant termination with pinned params) and bench bar (D9b,
  one-sided); FR-087-AC-1 carries the conditions.
- FND-006 **ACCEPTED/PLANNED** — first-of-kind shmem/latch risk budgeted in
  B2 (fixed-slot array, `_PG_init` wiring, abort callback); D4 notes the
  fixed slot array.
- FND-007 **RESOLVED** — D10 names the SPIRE dispatch port explicitly
  (detect-inside, return, then raise; CancelToken), avoiding the literal
  CHECK_FOR_INTERRUPTS-in-block_on trap; multi-hop propagation is the
  residual novelty, drilled in TC-046.
- FND-008 **RESOLVED** — TC-045 adds the append-only-beam invariant guard
  test.
- FND-009 **RESOLVED** — NFR-021 restates the envelope at shipped defaults
  (≈166 KB worst case) and sizes the mailbox cap against it; design doc
  updated.
- FND-010 **RESOLVED** — FR-085 pins the expansion budget as the
  authoritative bound (rounds derived); FR-086/FR-089 restated accordingly.
- FND-011 **NOTED** — refactor-risk assessment recorded; the preserved
  subtleties (kth select_nth_unstable, early-exit position, injection-point
  ordering) belong in the B0 task brief.
- FND-012 **RESOLVED** — cancellation moved to B1, mailbox lifecycle drills
  co-located with B2, B3 shrunk to the cross-cutting fault matrix (design
  doc milestone table).
- FND-013 **RESOLVED** — timeout semantics and mailbox cap pinned in
  FR-088/NFR-021 as part of this review reconciliation (recorded in ADR-086
  D4/D6 context and the design doc's resolved-items list), not left to
  implementation.
- FND-014 **NOTED** — no change needed; AC-1 reworded per SR-009 FND-007 to
  the runnable form.
