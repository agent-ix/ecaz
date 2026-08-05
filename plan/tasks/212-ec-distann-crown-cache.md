# Task 212: ec_distann Crown Cache

Status: **complete** (2026-08-02). Priority: P1.

P0 spec landed as
`spec/functional/distann/read/FR-089-distann-crown-cache.md` (hardened:
width pruning is an explicit measured arm with a population-complete
precondition; population timing + populated predicate defined; selection
digest keyed (epoch_fingerprint, capacity)); packet
`reviews/task-212/001-crown-cache-spec/` open. P1, the single-variable pruning
A/B, and the fused capacity sweep are complete in
`reviews/task-212/002-crown-cache-implementation/`. The pruning A/B activated
but pruned zero shards and showed no latency improvement. The fused capacity
matrix selected 2048 entries as the opt-in capacity; production defaults remain
off because the measured fused arms are explicitly seed-set-changing.

Entry gate: Task 210 merged. Spec phase (P0) can start immediately; the final
sizing sweep references Task 211's chosen law but does not wait on it.

## Why

Sharding the head costs one dedicated fan-out round trip before traversal
starts. At 100k that is ~+5% mean latency against the (non-conforming)
local-head referent; at 10k it is ~+8.6% (`reviews/task-210/006-zero-byte-head/`).
003a reviewer question 2 asked whether that cost is simply the price of the
invariant or worth a bounded coordinator-side cache. This task builds that
cache — under the conformance screen that keeps it from becoming the old
local head with a new name.

## What the crown is

A **fixed-capacity** coordinator navigation cache over a **subset** of head
landmarks, at per-landmark granularity: entries are `(vec_id, quantized code)`
— routing payload only, the same class as the TRAV-30 gateway copies, never a
full-precision vector. Capacity is a stated constant in entries/bytes,
independent of `N` **and of head size `C`**: when `C` grows past capacity, the
crown holds a coarser subset and hit quality degrades — coordinator memory
never grows. The crown never attempts to mirror the aggregate head.

Lifecycle: epoch-fingerprint-keyed, per-backend, populated lazily by bounded
batch RPCs from the owners (the owners stay the source of truth), rebuilt on
epoch flip, discarded and repopulated on a capacity GUC change (the
`72b5d5b52` staleness rule). No serve-time remote calls. **Rebuild-only by
design**: the head membership is frozen within an epoch (D10), so crown and
head cannot diverge; inserts reach new rows through owner graphs, not through
the head, and head refresh is an epoch-cadence concern, not a cache-mutability
concern.

Selection is **static/structural** in this task: a deterministic coarser
sample of the head (or its upper navigation layers), sized to capacity,
refusal-not-eviction, selection digest attested. Frequency-aware admission is
explicitly out of scope until a measured skew case justifies its
nondeterminism.

## Conformance screen (the FR-084 bright line)

- Bounded by a stated constant, enforced by refusal; capacity change discards.
- Codes only — nothing vector-shaped resident at the coordinator.
- Non-authoritative and rebuildable; a miss falls back to the full sharded
  head fan-out: identical results, one RTT slower, never a wrong answer
  (NFR-021 clause 4 — the distributed path is narrowed, never substituted).
- NFR-021 pre-registration as a bounded conforming structure; the crown's
  resident bytes are itemised on the coordinator storage row.
- **Activation counters from day one** (`crown_seeds_served`,
  `crown_fallbacks`), asserted non-zero in the candidate arm. Four mechanisms
  in Task 210 ran inert inside green suite runs; a fifth is not acceptable.

## Phases

- **P0 — spec first.** `/specify` the crown's FR/NFR (capacity semantics,
  selection determinism, fallback contract, conformance envelope), validated
  with `/spec-review`, before implementation.
- **P1 — structure + population + counters.** Crown build, epoch lifecycle,
  GUC (`ec_distann.crown_capacity`, default off), stats/observability
  surface.
- **P2 — width pruning.** Use crown scores to fan the head search only to
  promising owners; A/B at 10/50/100k. (The round-trip elimination itself is
  Task 213's fused hop — without 213 the crown's win is owner CPU and tail
  width.) The equal-seed-count A/B completed, with activation but zero shards
  pruned and no measured latency benefit at capacity 2048.
- **P3 — sizing sweep.** Complete: fused crown capacity 512/2048/4096 at
  10k/50k/100k. Capacity 2048 is selected for the opt-in fused configuration;
  1M+ remains deferred with Task 211's scale bound.

## Benchmark gate

Standard 10/50/100k A/B (`ecaz bench suite`, config in packet), crown off vs
on, one change per arm. Candidate must show non-zero activation counters,
`coordinator_resident_unsharded_bytes=0`, `outstanding_distribution_gap=none`,
and crown resident bytes within the stated capacity.
