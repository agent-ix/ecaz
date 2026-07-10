# ec_distann BatANN Mode: State-Passing Coordination and Milestone Definitions

Companion design doc for ADR-086 and the FR-084..FR-089 / NFR-021..NFR-022
spec batch (authored Task 173, branch `task-173-batann-specs`). This document
is the normative home of the milestone definitions B0–B4 that the specs and
test matrix reference. It reopens ADR-085 D4 ("BatANN baton passing rejected
for now"), whose reopen trigger is the hop-round RTT share of multinode p50;
this program specs the mechanism so the M2/M4 measurements can exercise it
directly rather than gate on a projection.

Paper: "Passing the Baton: High Throughput Distributed Disk-Based Vector
Search with BatANN" (arXiv:2512.09331). Reference results: 1.44–2.09× the
throughput of DistributedANN at 1B/10 servers at 0.95 recall@10, mean latency
< 3 ms on plain TCP, achieved by forwarding the entire query state to the
server owning the next frontier candidate instead of a coordinator
round-trip per hop.

## Architecture summary

BatANN mode is a **coordination strategy over the existing ec_distann index**
— same global Vamana graph, same record format (FR-076), same hash placement
(FR-078), same epochs (FR-082). Nothing on disk changes; only who advances
the beam. A session GUC selects one of three modes per query:

- `coordinator` (default) — today's FR-081 loop: the coordinator holds the
  beam and issues one `ec_distann_expand_nodes` call per owning node per hop
  round.
- `batann_stack` — when the entire top-BW frontier is remote, the current
  node serializes the full query state and synchronously calls
  `ec_distann_relay_search` on the node owning the best unexpanded
  candidate; results unwind the nested call chain back to the coordinator.
- `batann_direct` — same forwarding, but intermediate nodes do not hold the
  chain: the terminal node delivers the final state straight to the
  coordinator via `ec_distann_deliver_result` into a shared-memory result
  mailbox that the coordinator backend waits on.

Three building blocks:

1. **State seam.** The FR-081 loop locals become an explicit, serializable
   `DistannBeamState` (beam entries `(vec_id, code_dist, expanded)` — the
   beam is append-only today, so the entry array doubles as the visited set;
   hits `(vec_id, exact_dist)`; counters; budgets). A shared
   `distann_local_drain(state) -> Complete | Handoff(owner)` implements paper
   Algorithm 2: expand **all locally-owned top-BW frontier candidates**
   before handing off; hand off only when the whole top-BW frontier is
   remote, to the owner of the best unexpanded candidate. Coordinator mode is
   re-expressed over the same state so all modes share convergence (D9
   early-exit), dedupe, and the BW×H budget logic; the budget travels in the
   state, so NFR-019 holds regardless of which node expands.
2. **Relay transport.** The existing per-backend pooled libpq/tokio transport
   (`remote_transport.rs`, pool keyed `(conninfo, node_id)`) is generalized so
   data-node backends hold pools too — full mesh, since every node has the
   roster and can compute placement. Stack mode is a synchronous nested SQL
   call; direct mode forwards send-and-abandon (issue the downstream relay
   statement, do not await; connection marked busy-until-drained) so
   intermediate backends free immediately.
3. **Return paths.** Stack: the chain unwinds (depth d holds d+1 backends —
   bounded by `ec_distann.relay_max_depth`, default min(H, 16)). Direct:
   fixed-slot shmem mailbox keyed by never-reused 64-bit query_ids +
   waiting-backend latch; delivery is at-most-once (first delivery wins,
   delivery rights travel with the state — a node that confirmed a handoff
   never delivers), and a node failing before a confirmed handoff delivers
   its error itself, so the coordinator timeout (a classified error, never
   a silent rerun) only covers black holes. Slot exhaustion falls back
   transparently to coordinator mode. Depth exhaustion returns the state
   marked incomplete and the coordinator **resumes it in the
   coordinator-mode hop loop, terminally — no further handoffs per attempt**
   (hybrid fallback; `relay_max_depth = 0` degenerates to coordinator
   mode). The FR-083 delta-buffer merge stays a coordinator-side
   post-search step in every mode, like materialization.

### Relay state wire format (v1 sketch, normative form in FR-085)

`DISTANN_RELAY_STATE_V1`, versioned bytea: magic + version; flags (return
mode, incomplete); epoch + 16-byte epoch fingerprint (validated on **every**
hop before any use beyond the header, same discipline as FR-079); index name
(regclass-castable, the existing cross-node index handle); raw query vector
(the quantized `DistannPreparedQuery` is recomputed per node from local
codebooks — sound because the fingerprint attests both sides run the same
epoch/codebooks); params (beam_width, effective_top_k, hop_rounds_remaining,
relay_depth_remaining, code_threshold); beam entries; hits (no heap_tids ever
travel — heap_tids are node-local per FR-079); counters incl. relay counters
(relay_hops, state_bytes max/total, depth trail); direct-mode return address
(coordinator_node_id + query_id; conninfo resolved from the roster). Unknown
version → non-retriable error; structurally invalid states (duplicate beam
vec_ids, budgets above session maxima, oversized arrays) are rejected too
(D11 distrust posture). Size envelope: beam ≤ seeds + expansions×R × 13 B,
hits ≤ expansions × 12 B — ≈166 KB worst case at shipped defaults (BW=4,
H=100, R=32, budget fully spent), typically far smaller under early-exit;
bound and mailbox-cap sizing stated in NFR-021.

### Hazard analysis (details pinned in ADR-086)

- **Occupancy, not deadlock (stack).** A→B→A re-entry opens a *new* backend
  on A; nothing ever waits on its own backend, so no lock cycle. Worst-case
  backends per query = relay_max_depth + 1; `max_connections` sizing guidance
  = concurrent_queries × (relay_max_depth + 1) worst case (NFR-021).
- **Cancellation.** Today's transport `block_on` lacks
  `CHECK_FOR_INTERRUPTS`; relay makes one long await, so the transport gains
  interrupt-sliced awaits + downstream CancelToken propagation; cancels and
  statement timeouts unwind hop-by-hop (D10).
- **Direct-mode forwarding.** A SQL function cannot "ack then keep working";
  the freeing move is on the caller: send the downstream statement without
  awaiting the response (PostgreSQL executes a received statement regardless
  of whether the client reads the result). Flush-before-return mechanics are
  a named B2 spike; fallback is "direct-lite" (synchronous forward acks,
  mailbox return only), which still measures the mailbox half.
- **Materialization fix.** `custom_scan.rs:fetch_remote_payloads` treats a
  locally-owned hit with INVALID ctid as a structural fault; under relay,
  coordinator-owned vec_ids expanded on other nodes arrive that way
  legitimately → re-resolve through the local directory (FR-087 AC).
- **Head index.** Only the coordinator does FR-080 head descent; relay nodes
  receive a seeded state (they still load the cache entry for directory +
  codebooks, as the expand endpoint already does).
- **Hash-placement caveat (D7).** The paper's 10–30% inter-partition hop
  rate relies on locality-preserving graph partitioning; ec_distann places by
  hash, so the expected relay rate approaches 100% of hop rounds. The bench
  gate must measure and report relay-rate-per-hop-round explicitly;
  locality-aware placement is a named out-of-scope follow-up with its own
  reopen trigger.

### Config surface (new GUCs, FR-084)

- `ec_distann.coordination_mode` = `coordinator` | `batann_stack` |
  `batann_direct` (enum, Userset, default `coordinator`).
- `ec_distann.relay_max_depth` (int; default = min(effective hop_rounds,
  16); 0 ≡ coordinator mode).
- `ec_distann.relay_wait_timeout_ms` (direct-mode mailbox wait; default
  10000 ms).
- Debug GUCs (all default off, NFR-020 posture):
  `ec_distann.debug_fail_relay_depth` (fail at depth N),
  `ec_distann.debug_hold_relay_depth` (stall at depth N for occupancy /
  mid-drain-kill drills), `ec_distann.debug_relay_trace_notice` (per-drain
  decision trace).
- Profile notice / EXPLAIN / results.jsonl counters (normative list in
  FR-084): relay_hops, relay_depth_max, relay_depth_histogram,
  state_bytes_max, state_bytes_total, drains_executed, head_descents,
  handoffs_per_node, fallback_resumed, relay_journeys. Relay sessions tag
  `application_name = 'ec_distann_relay'`.

## Milestone definitions (normative)

| ID | Name | Delivers | Exit criterion |
|----|------|----------|----------------|
| **B0** | State seam | `DistannBeamState` + `distann_local_drain` extraction (pure refactor, coordinator mode re-expressed over it), `relay_state.rs` serde (FR-085), GUC registration (FR-084 surface), local-only `ec_distann_relay_search` (single-node form, no transport) | Existing FR-081 unit/pg tests green over the refactor; TC-045 round-trip/version/structural-bounds/fingerprint suite green; GUC default/invalid drills; single-node relay identity: `ec_distann_relay_search` on one node reproduces `collect_distann_hits` results exactly |
| **B1** | Stack mode | Transport relay wiring for the B0 endpoint (node→node calls), `coordination_mode`/`relay_max_depth` GUCs (FR-084), depth budget + hybrid coordinator-mode resume (FR-089), batann-scoped `fetch_remote_payloads` local-hit fix (FR-087), **interrupt-sliced awaits + downstream CancelToken propagation** (shared-path enabler — also fixes coordinator-mode uncancellability; ported from the SPIRE dispatch pattern) | TC-046: 2/3-node loopback fixture recall parity vs coordinator mode (top-k identity under convergence-dominant termination); `relay_max_depth=0` ≡ coordinator mode; depth-exhaustion resume drill; cancel-chain drill; relay counters emitted; **kill-check checkpoint**: informational stack-vs-coordinator latency + relay-rate row, with a recorded proceed/de-scope verdict gating B2 (de-scope = defer direct mode) |
| **B2** | Direct mode | **Pre-implementation timeboxed flush spike** (send-and-abandon on tokio-postgres is likely unobtainable — lazy futures, no flush signal; direct-lite is the probable shipped form), then: fixed-slot shmem mailbox + monotonic query_id allocator + `ec_distann_deliver_result` (FR-088), `relay_wait_timeout_ms`, mailbox lifecycle drills (slot exhaustion fallback, duplicate/orphan delivery, timeout, abort cleanup) | TC-047 happy paths + mailbox lifecycle drills; latch wakeup; stack ≡ direct results for identical query/budgets; spike verdict recorded in ADR-086 |
| **B3** | Faults + lifecycle | Cross-cutting fault matrix: epoch-mismatch mid-chain restart, link-failure teardown, killed terminal node, connection busy-until-drained hygiene, `debug_fail_relay_depth` / `debug_hold_relay_depth` matrix | TC-046/TC-047 fault matrices green; NFR-021 drill evidence (no orphaned backends/undrained connections after cancel/timeout, per instance across all nodes); restart-once on mid-chain epoch mismatch |
| **B4** | Bench gate | Suite-runner coordination-mode axis + relay-counter emission in results.jsonl | NFR-022 packet: pre-registered 3-way mode matrix (coordinator / batann_stack / batann_direct) at 10k/50k/100k, 3-worker, NFR-017 protocol/host; distinct_recall@10 parity bar met in all modes; latency p50/p95 per mode; D7 relay-rate finding recorded |

Milestone→task mapping: spec authoring = 173 (this lane), B0=174, B1=175,
B2=176, B3=177, B4=178.

## Design invariants worth restating

- BatANN modes change **who advances the beam**, never what is expanded:
  results still come only from expanded records, the BW×H cap is global
  across all drains of one attempt, and the FR-081 convergence rule is
  evaluated against the carried state wherever it lives.
- The epoch fingerprint gates every hop (relay and delivery), so a republish
  mid-flight costs exactly one restart, matching FR-082-AC-2.
- heap_tids never travel; materialization stays coordinator-driven after the
  search completes, in every mode.
- Mode is a per-query session GUC over one index/epoch — the bench matrix
  flips modes per statement with no rebuilds; a single-node roster or
  `relay_max_depth = 0` makes every mode result-equivalent to coordinator
  mode.
- Result-equivalence bar (D9): recall parity is the AC; top-k identity on the
  deterministic fixture is the evidence form (traversal order differs from
  coordinator mode by construction — Algorithm 2 drains local candidates
  eagerly).

## Reuse map

- `src/am/ec_distann/scan.rs` — `distann_orchestrated_search` loop locals →
  `DistannBeamState`; drain loop extracted from the hop-round body.
- `src/am/ec_distann/remote_transport.rs` — pooled transport reused verbatim
  for node→node relay calls; gains interrupt-sliced awaits, CancelToken
  propagation, busy-until-drained marking.
- `src/am/ec_distann/remote_endpoint.rs` — endpoint pattern (fingerprint
  validation, SQLSTATE classes) for `ec_distann_relay_search` /
  `ec_distann_deliver_result`.
- `src/am/ec_distann/routine.rs` — `collect_distann_hits` gains the mode
  dispatch, covering both the amgettuple and CustomScan read paths.
- `src/am/ec_distann/custom_scan.rs` — mailbox wait sits behind
  `run_search_and_build_outputs`; `fetch_remote_payloads` local-hit fix.
- `src/am/ec_distann/options.rs` — GUC registration precedent.
- `crates/ecaz-cli/.../distann_multicluster.rs` — the loopback multinode
  fixture runs TC-046/TC-047 drills; suite runner gains the mode axis for B4.

## Open items tracked to milestones

- Send-and-abandon flush guarantee → pre-B2 timeboxed spike; direct-lite is
  the recorded (and probable) fallback; FR-088's occupancy language is
  already scoped per variant and the gate packet records which ran.
- `relay_max_depth` default (= min(H, 16), pinned by ADR-086 D6 after
  review) is provisional until B4's relay-rate measurement.
- Mailbox inline payload cap: pinned to the computed NFR-021 envelope
  (≈166 KB worst case at shipped defaults), oversize → delivered error; no
  DSM overflow in v1. B2 implements this form.
- B4 merge prerequisites (ADR-086 Measurement Requirements): task-165 lane
  merged or B-lane residency declared; relay-counter suite step kind landed
  as its own commit (the `distann-pipeline` step kind cited by
  NFR-017/TC-044 does not exist yet); task-172 protocol pinned by a landed
  packet.
- Locality-aware placement (restoring the paper's 10–30% relay rate) is
  out of scope; reopen trigger recorded in ADR-086 D7. Throughput claims
  are likewise out of scope (no inter-query balancing analogue); reopen
  trigger in ADR-086 Alternatives.
- Resolved by spec review (2026-07-09, spec/reviews/batann/): direct-mode
  wait timeout = classified error, never a rerun; slot exhaustion =
  transparent coordinator-mode fallback; deepening re-runs = new attempts
  counted in `relay_journeys`; resume is terminal (no re-relay);
  query_ids never reused; endpoint EXECUTE revoked from PUBLIC (D11).
