---
type: ADR
id: ADR-086
title: "ec_distann: BatANN State-Passing Coordination Mode (reopens ADR-085 D4)"
status: PROPOSED
impact: Adds a query-time coordination strategy to ec_distann — relay/baton state passing between data nodes — alongside the default central-coordinator loop. Governs FR-084..FR-089 and NFR-021..NFR-022. Reopens and supersedes ADR-085 D4 (baton passing "rejected for now"); leaves the on-disk format (FR-076), placement (FR-078), and epoch model (FR-082) untouched.
date: 2026-07-09
---
# ADR-086: ec_distann — BatANN State-Passing Coordination Mode

## Context

ADR-085 D4 rejected BatANN-style baton passing "for now" with a reopen
trigger: an M2 measurement showing hop-round RTT ≥ 50% of multinode p50 at
gate-relevant BW/H. The coordinator loop (FR-081) pays one per-node RTT per
hop round by construction; with hash placement nearly every hop round crosses
the network, so the H×RTT term is the dominant structural latency floor of
the multinode read path.

The BatANN paper (arXiv:2512.09331, "Passing the Baton") measures the
alternative directly against a DistributedANN re-implementation: instead of a
request–reply round-trip per hop, the node that owns the next frontier
candidate receives the **entire query state** (query vector, beam, visited
set, accumulated full-precision results, budgets — 4–8 KB at paper scales)
and continues beam search locally, expanding everything it owns before
handing off again (paper Algorithm 2); the last active server returns the
result. On 1B points / 10 servers at 0.95 recall@10 this yields 1.44–2.09×
DistributedANN's throughput with mean latency < 3 ms over plain TCP.

Rather than gate on the D4 RTT projection alone, this program specs the
mechanism as a **query-time coordination mode** so the M2/M4-class
measurements can run coordinator and relay modes head-to-head on the same
index, same epoch, same corpus. The operator directive: default mode remains
the DistANN central coordinator; BatANN mode enables direct query passing
between data nodes; both result-return variants (unwind the call chain vs
deliver direct to the coordinator) are specced and measured; nodes are fully
meshed with pooled connections; a max-depth setting bounds uncoordinated
relaying; all modes are benchmarked.

Companion design doc (normative home of milestones B0–B4):
`plan/design/batann-state-passing-coordination.md`.

## Decision

Add a BatANN state-passing coordination mode to ec_distann, selected
per-query by a session GUC, implemented as a relay protocol over the existing
pooled node-to-node transport, with two result-return sub-modes (stack and
direct), a relay depth budget with hybrid coordinator-mode resume, and a
pre-registered three-way mode benchmark gate. No on-disk change; no build
change; scans in every mode traverse the same published epoch.

## Sub-Decisions

- **D1 — Mode surface is a Userset session GUC**
  (`ec_distann.coordination_mode` = `coordinator` (default) | `batann_stack`
  | `batann_direct`), plus `ec_distann.relay_max_depth` and
  `ec_distann.relay_wait_timeout_ms` (FR-084). A reloption is rejected: the
  mode is a query-time strategy over one index/epoch, and the NFR-022 bench
  matrix must flip modes per statement without rebuilds. Coordination mode
  never participates in the epoch fingerprint.
- **D2 — Relay-state wire format v1** (FR-085): versioned, length-prefixed
  bytea carrying flags, epoch + 16-byte epoch fingerprint, index name, raw
  query vector, budgets, beam entries `(vec_id, code_dist, expanded)` — the
  append-only beam doubles as the explicit visited set — hits
  `(vec_id, exact_dist)`, counters, and (direct mode) the return address.
  The quantized query (`DistannPreparedQuery`) is **recomputed per node**
  from local codebooks; the epoch fingerprint attests both sides share
  codebooks/metadata, so recompute is sound and keeps the state small.
  Unknown version → non-retriable error (NFR-016 discipline: reject, never
  skip). heap_tids never travel (they are node-local per FR-079).
- **D3 — Stack return mode = nested synchronous SQL unwind** (FR-087): a
  relay is a synchronous `ec_distann_relay_search(index, state)` call from
  the current node to the owner of the best unexpanded frontier candidate;
  the terminal state unwinds the call chain to the coordinator's original
  blocked call. Chain depth d occupies d+1 backends and d pooled
  connections — bounded by D6 and stated in NFR-021.
- **D4 — Direct return mode = shared-memory result mailbox + latch, with
  send-and-abandon forwarding** (FR-088): the coordinator registers a
  query_id slot in a fixed-size shmem mailbox (fixed slot array, not a
  hash) and waits on its latch; intermediate nodes forward the state by
  issuing the downstream relay statement **without awaiting the response**
  (the connection is marked busy-until-drained; PostgreSQL executes a
  received statement regardless of whether the client reads the result),
  freeing their backend; the terminal node calls
  `ec_distann_deliver_result(query_id, fingerprint, status, state)` at the
  coordinator host, landing in a fresh backend that fills the slot and sets
  the latch. Delivery is **at-most-once with a timeout backstop**:
  query_ids are allocated from a 64-bit monotonic per-host shmem counter
  and never reused within a postmaster lifetime; the first delivery to a
  slot wins and later deliveries for the same query_id are dropped with a
  WARNING; **delivery rights travel with the state** — a node that has
  confirmed a downstream handoff relinquishes the right to deliver (it must
  not race its own downstream chain with an error), and a node whose
  forward outcome is indeterminate delivers nothing (the coordinator
  timeout backstops). A node that fails **before** a confirmed handoff
  delivers the failure — with its FR-079 classification — to the mailbox
  itself, so the timeout covers only true black holes. Rejected
  alternative: "return the result back through the chain's first
  connection" — with SQL-function relays the first call's return IS the
  chain unwind, i.e. it collapses into D3. The send-and-abandon flush
  guarantee is a **pre-B2 timeboxed spike** (tokio-postgres futures are
  lazy and expose no outbound-flush signal, so this may be unobtainable on
  the stock driver); the recorded fallback is **direct-lite** (synchronous
  forward acks, mailbox return only — stack-mode occupancy but no return
  trip through the chain). FR-088's backend-freeing language is scoped to
  the send-and-abandon variant; the NFR-022 gate packet records which
  variant ran.
- **D5 — Transport generalization, not a new transport**: the per-backend
  thread-local pool keyed `(conninfo, node_id)` (FR-079 / NFR-014 posture) is
  reused verbatim on data-node backends for relay calls — full mesh follows
  from every node holding the roster and computing placement locally
  (FR-078). New pool discipline: busy-until-drained marking and a cap on
  abandoned in-flight sends (D4), and interrupt-sliced awaits (D10).
- **D6 — Relay depth budget with hybrid resume** (FR-089):
  `ec_distann.relay_max_depth` decrements per handoff; default =
  **min(effective H, 16)** (H alone is unsafe: the shipped hop-round
  default is 100, which would permit a 101-backend worst case per query;
  provisional until the B4 relay-rate measurement). On exhaustion the
  current node marks the state incomplete and returns/delivers it; the
  coordinator **resumes the same state in the coordinator-mode hop loop**
  with the remaining expansion budget, and the resume is **terminal** — no
  further handoffs occur within the attempt (total handoffs per attempt are
  therefore bounded by relay_max_depth). `relay_max_depth = 0` degenerates
  exactly to coordinator mode — a required equivalence test.
- **D7 — Hash-placement relay-rate caveat, measured not assumed**: the
  paper's 10–30% inter-partition hop rate depends on locality-preserving
  graph partitioning; ec_distann's placement is hash (FR-078,
  load-balance-only), so the expected relay rate approaches one handoff per
  hop round. NFR-022 mandates reporting relay-rate-per-hop-round, relay-hop
  counts, and state-bytes so the mode comparison is honest about this.
  Locality-aware placement is out of scope for this program; reopen trigger:
  B4 evidence that relay mode's latency win is materially capped by relay
  rate (e.g. relay transfers ≥ 50% of relay-mode p50).
- **D8 — The BW×H budget travels in the state** (FR-085/FR-086): every
  node's drain decrements the same global expansion budget, preserving
  NFR-019 verbatim in all modes ("results only from expanded records" and
  the convergence early-exit rule are properties of the state, not of the
  node holding it).
- **D9 — Result-equivalence bar (two distinct bars, do not conflate)**:
  (a) the **fixture bar** — identical final top-k on the deterministic
  multinode fixture, valid only under convergence-dominant termination
  (generous H, `early_exit` counter-asserted, seeded corpus with a
  deterministic distance tie-break at the k boundary, fixed BW/H/k); relay
  traversal order differs from coordinator mode by construction (Algorithm
  2 drains locally-owned candidates eagerly), so when the expansion budget
  binds, a different top-k is legitimate and the identity assertion does
  not apply; (b) the **bench bar** — distinct_recall@10 ≥ coordinator mode
  − 0.001 (one-sided) at matched BW/H in every NFR-022 cell. FR-087-AC-1
  tests bar (a); NFR-022 gates on bar (b).
- **D10 — Cancellation/timeout authority is the coordinator statement**: the
  transport's blocking awaits become interrupt-sliced with downstream
  cancel propagation, porting the existing SPIRE dispatch pattern
  (`ec_spire/coordinator/remote_candidates/dispatch.rs`: poll the
  interrupt/timeout indicators, return out of `block_on`, then raise —
  never raise inside the runtime — and propagate via
  `tokio_postgres::CancelToken`). A coordinator cancel or any intermediate
  statement timeout unwinds a **stack** chain hop-by-hop. **Direct-mode
  cancel is coordinator-local**: after send-and-abandon there is no
  connection chain to propagate through, so an in-flight remote drain runs
  to completion (bounded by BW×H and the depth budget), then delivers to a
  freed slot and is dropped; such bounded residual work is not an orphan
  for NFR-021 purposes, provided all backends quiesce. The mailbox slot is
  freed via the transaction-abort callback; late/orphan deliveries to a
  freed or unknown query_id are dropped with a WARNING. Because the
  transport fix is shared, it also makes today's coordinator-mode
  `ec_distann_expand_nodes` calls cancellable (a pre-existing FR-079-land
  gap): this shared-path behavior change lands at **B1** as its own slice,
  benefiting every mode.
- **D11 — Endpoint authorization posture**: `ec_distann_relay_search` and
  `ec_distann_deliver_result` SHALL have EXECUTE revoked from PUBLIC and
  granted to the roster operator role only. The epoch fingerprint attests
  epoch identity, not caller identity (it is computable by any roster
  participant), and query_ids must not be treated as capabilities. Within
  the research deployment posture (NFR-014 lift: private network,
  operator-managed conninfo) this role gate is the accepted control;
  per-query capability tokens in the state are the named escalation if the
  posture ever hardens. Structural distrust of the payload is separate and
  mandatory: FR-085 requires bounds/consistency validation of every
  received state regardless of caller.

## Rationale

- The H×RTT structural floor is the one term of the multinode read path that
  the coordinator loop cannot remove; state passing removes the per-hop
  return leg and lets whichever node owns the frontier advance the query
  with local reads (paper §4, §6.2–6.6).
- Making the mechanism a per-query GUC turns ADR-085 D4's reopen trigger
  from a projection into a direct A/B: NFR-022's three-way matrix measures
  coordinator vs stack vs direct on identical corpus/epoch/host.
- The two return sub-modes bracket the design space: stack is simple and
  transactionally boring but holds a backend per chain hop; direct frees
  intermediate backends at the cost of a shmem mailbox and a delivery
  endpoint. Measuring both prices the occupancy/complexity trade.
- The hybrid depth-exhaustion resume keeps BatANN mode strictly a
  performance strategy: completeness and budget guarantees (FR-081/NFR-019)
  hold across the relay/coordinator splice, and the degenerate depth-0 case
  gives a cheap always-on correctness anchor.

## Measurement Requirements

- NFR-022 pre-registered three-way mode matrix (coordinator / batann_stack /
  batann_direct) at 10k/50k/100k under the NFR-017 protocol and host, with
  the D9 bench bar, per-mode latency p50/p95, and the FR-084 relay counters
  (incl. `relay_depth_histogram`, `state_bytes_max/total`, and the
  pre-registered relay-rate derivation) emitted to results.jsonl.
- **B1 kill-check gating B2**: B1's exit includes an informational
  stack-vs-coordinator latency + relay-rate checkpoint on the 2/3-node
  fixture; a recorded proceed/de-scope verdict (de-scope = defer direct
  mode, keep stack mode for the B4 record) is required before B2 spends on
  the mailbox. Rationale: coordinator mode expands the whole top-BW
  frontier across owners in one grouped parallel RTT per round, while
  relay mode serializes owner visits — under hash placement the structural
  comparison can already be negative at 2–3 nodes.
- D7 evidence row: measured relay rate under hash placement, cited when
  deciding the locality-aware-placement follow-up.
- Pre-B2 timeboxed flush spike verdict (send-and-abandon vs direct-lite)
  recorded against this ADR before B2 implementation starts.
- Merge prerequisites for B4 (state them in the gate packet): the
  task-165 distann lane merged to main (or the B-lane explicitly kept on
  that branch), the relay-counter-emitting suite step kind landed as its
  own commit per the FR-038 suite rule (the `distann-pipeline` step kind
  named by NFR-017/TC-044 does not exist yet), and the task-172
  real-multinode protocol pinned by a landed packet.

## Alternatives Considered

- **Keep coordinator-only and wait for the D4 trigger measurement**:
  rejected — the trigger measurement itself is cheapest to obtain with the
  relay mode implemented behind a default-off GUC, and the paper's deltas
  justify the spec investment now.
- **Chain-connection result return** (terminal result rides back through the
  first connection): rejected per D4 — indistinguishable from stack mode
  under SQL-function relays.
- **Relopt-pinned mode**: rejected per D1 — kills per-statement A/B and
  buys no safety (the mode does not affect on-disk state).
- **Carrying the quantized query in the state**: rejected per D2 — codebook
  identity is already attested by the epoch fingerprint; recompute keeps
  state size down and avoids a second codec-versioning surface.
- **Locality-aware placement in this program**: deferred per D7 — it changes
  FR-078's load-balance-only contract and the build; it is the natural
  follow-up if B4 shows the relay rate caps the win.
- **Non-adopted BatANN paper concepts** (named so their absence is a
  decision, not an oversight): (a) *inter-query balancing / multiple
  query-states per worker thread* — the mechanism behind the paper's
  headline 1.44–2.09× **throughput**; per-backend PostgreSQL execution has
  no equivalent, so NFR-022 gates on latency and recall only, throughput is
  explicitly out of gate scope, and inter-query balancing is the reopen
  trigger for any future throughput claim; (b) *any-node query origination
  with a replicated head index* — here only the coordinator seeds (FR-080);
  deferred, not needed for the mode A/B; (c) *per-node query-embedding
  caching* — moot: the query travels in the state and the quantized form is
  recomputed per D2; (d) *dedicated async messaging transport
  (ZeroMQ-style)* — rejected per D5, the pooled libpq transport is the
  contract; (e) *W=64 pipeline width* — BW stays the FR-081 session GUC at
  its own defaults; NFR-022 may add an informational BW-sensitivity row but
  the gate does not sweep the paper's W.
