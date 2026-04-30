# Task 29 DiskANN — Merge-Readiness Review

Reviewer: opus-review
Branch: `task29-diskann-initial-tuning` @ `5772a33d`
Scope: Task 29 + Task 29a — recall fix (binary-sidecar prefilter) and
latency fix (heap frontier + early-stop). Reviewed against the chain
of recommendations from `review/11095-task29-diskann-pgvectorscale-comparison/`
and the plan in `plan/tasks/29a-diskann-binary-sidecar-prefilter.md`.

## Verdict

**Code quality is merge-ready, but two follow-ups are required
before landing per direction from review:**

1. **Task 29b — vacuum consistency + cleanup.** Scan-path now uses
   binary sidecar; vacuum repair still uses grouped-PQ. Same kind
   of recall ceiling reappears in repaired neighbor slots. Plus the
   GUC doc-string update, missing pgrx test, SIMD verification on
   `hamming_xor_popcount`, and small code-shape extracts.
2. **Task 29c — build performance.** 492 s for 10k × 1536-d on
   single-process build is the open question. Per direction:
   measure first, exhaust within reason, compare against `ec_hnsw`
   and pgvectorscale references, then decide whether single-process
   is landable or whether parallel build is the next ask.

The implementation that exists matches what was recommended in
`review/11095`, the measured recall and latency deltas land
cleanly, the focused PG18 callback smoke passes (19/19), `cargo
clippy --features pg18 -D warnings` is clean, and the 28
`am::ec_diskann::scan` unit tests pass at HEAD. The branch is in
good shape — the gates are scope-completion (29b vacuum
consistency, 29c perf measurement), not code-quality issues.

| Goal | Target | Result | Source |
|---|---|---|---|
| Recall@10 at L=200 | ≥ 0.97 | **0.9970** | `11096`, `11099` |
| Recall@10 at L=64 | ≥ 0.97 | **0.9965** | `11096` |
| L=800 mean latency | improve over 247 ms | **68.9 ms** (3.6×) | `11098` |
| L=200 mean latency | no regression | **53.2 ms** (1.3× faster) | `11098` |
| pg_test coverage | callback smoke green | **19/19 passed** | `11099` |
| Storage @ 10k × 1536d | < 13 MiB hnsw ref | **4.7 MiB** (494 B/row) | `11099` |
| `cargo clippy` | clean | clean | reviewer re-ran |
| Unit tests | green | 28/28 | reviewer re-ran |

Plus the diagnostic that started this all (`11091`/`11094`): query
`10001` now matches exact 10/10, and IDs `9717`/`7782` enter the
binary-sidecar frontier at ranks 25 / 47 — both inside the default
`rerank_budget = 64`. The specific symptom that motivated the chain
of probes is gone.

## Comparison vs `ec_hnsw` reference on the same corpus

| | recall@10 | mean lat (L=200/ef=200) | p99 | index size | bytes/row |
|---|---|---|---|---|---|
| ec_diskann (sidecar) | **0.9970** | 53.2 ms | 90.1 ms | **4.7 MiB** | **494 B** |
| ec_hnsw (reference) | 0.9700 | 35.3 ms | 49.1 ms | 13.0 MiB | 1366 B |

DiskANN now beats HNSW on recall by ~3 percentage points and on
storage by 64%, at the cost of ~50% higher mean latency. That's the
expected DiskANN/HNSW trade-off shape for a disk-first design and
matches the Task 29 charter's "credible path beyond the HNSW
memory-resident ceiling" gate.

## Code review

### `src/am/ec_diskann/scan_query.rs` — new prefilter helpers

`pack_query_sign_bits` and `hamming_xor_popcount` (lines 111-134).
Both small, both unit-tested (`cr_009`, `cr_010`). Implementations
mirror the index-side packing in `quant::rabitq::sign_words_from_rotated`
exactly, which is the only correctness invariant that matters here.

One small note: `hamming_xor_popcount` uses the naive
`iter().zip().map(...).sum()` shape. pgvectorscale uses an
explicit-length match-arm unroll (`distance/mod.rs:255-323`) to give
the compiler exact size hints for SIMD codegen. At our fixed 24 u64
loop length on 1536-d corpora, the compiler should auto-vectorize
this fine, but it's worth a `cargo asm` check at some point to
confirm `popcntq` lands inside a tight loop. Not a blocker — the
measured per-visit cost is already low enough that the scan is
dominated by page reads, not popcount math.

### `src/am/ec_diskann/options.rs` — new `PrefilterKind` GUC

`ec_diskann.prefilter_kind` enum GUC with `auto` / `binary_sidecar` /
`grouped_pq` values. `Auto` falls back to `grouped_pq` when the
sidecar payload flag is clear, and `BinarySidecar` errors loudly when
the flag is missing. Sensible default behavior.

The doc string says "Diagnostic override used for Task 29a A/B
measurement." If this GUC is kept beyond the landing slice (i.e. as
an emergency rollback knob), revise that string to reflect
production intent. If it's purely diagnostic and gets removed when
the grouped-PQ path is deleted, leave the string and tag it for
removal in 29b. Either way, decide before merge.

### `src/am/ec_diskann/routine.rs` — prefilter selection

The `amrescan` body now branches between two prefilter closures
based on `use_binary_sidecar`. The branching is repeated in two
places (the optional codebook setup and the closure passed to
`vamana_scan_with`) which means `scan::vamana_scan_with` is invoked
in two near-identical `if/else` arms. This is mechanically correct
but a bit verbose. Worth extracting into a helper later, but doesn't
need to block the merge — the code is auditable as-is.

The error message at line 606
(`"ec_diskann.prefilter_kind=binary_sidecar requested but index has
no binary sidecar"`) is clear and actionable. Good.

### `src/am/ec_diskann/scan.rs` — heap frontier + early-stop

This is the meatier change. Final shape (after both `11097` and
`11098`):

- `next_heap: BinaryHeap<Reverse<ScanCandidate>>` — min-heap of
  unvisited candidates. ✓
- `entries: HashMap<ItemPointer, FrontierEntry>` — caches decoded
  neighbor list at discovery time so expansion doesn't re-read the
  picked tuple. ✓
- `visited_best: Vec<ScanCandidate>` — sorted ascending by score,
  via `partition_point` insert. ✓
- Early stop:
  `visited_best.len() >= list_size && next >= visited_best[L-1]`. ✓
  Matches pgvectorscale's `head.0 >= *node_at_pos` shape from
  `graph/mod.rs:158-164`.

Two minor observations:

1. **`peek_next_active` followed by `pop_next_active` redoes the
   stale-entry skim** (line 306). The peek already pops invalidated
   heap heads to expose the active one; the pop then calls peek
   again before popping the now-active head. The second peek is
   O(1) because the heap top is fresh, but it's a slightly clumsy
   abstraction. Either inline the pop into the early-stop check, or
   change `peek_next_active` to return both the candidate and a
   "remove" closure. Minor; doesn't matter for correctness.

2. **`visited_best.insert(idx, candidate)` is O(L)** per call due
   to `Vec::insert`'s memmove. At L=200 that's ~200 word memcpys
   per visit, which is 40× cheaper than the old per-visit
   `frontier.sort()` and not worth changing — but if L ever moves
   up to thousands, switch to a `BTreeSet` or a bounded skip list.
   Tracked here so it doesn't get forgotten.

Both observations are flagged for the cleanup task, not for landing.

### Test coverage

- 28 Rust unit tests covering the scan path pass. The pre-existing
  SC-001..SC-017 cover greedy descent + rerank end-to-end with
  synthetic prefilters; the new `cr_009`/`cr_010` cover the sign-pack
  and Hamming helpers. The recall-relevant SC-007 (synthetic L2
  brute-force comparison) and SC-010 (build + greedy descent
  end-to-end) both still pass at the new heap-frontier shape, which
  is the most direct unit-level evidence that the early-stop logic
  matches the brute-force ranking.
- 19/19 `pg_test_ec_diskann_*` callback tests pass at `b1cee686`.
  Coverage spans build, ordered scan, insert/duplicate handling,
  planner gating, session GUC override, and vacuum repair.

**Gap I'd flag (non-blocking)**: there is no pgrx integration test
for the `ec_diskann.prefilter_kind` GUC override path. The
session-list-size GUC has `pg_test_ec_diskann_session_list_size_override_changes_scan_width`;
the prefilter GUC doesn't have an analogous test. If the GUC is kept
beyond Task 29a (see options.rs note above), add one. If it's slated
for removal in 29b alongside the grouped-PQ path, fine to skip.

## Bench results sanity check

I cross-referenced the recall-and-latency tables in `11096`,
`11098`, and `11099` against each other. The numbers are
self-consistent: the L=800 mean trajectory `247 → 108 → 69 ms`
across `11096` (sidecar baseline, pre-heap) → `11097` (heap
frontier) → `11098` (early-stop) makes physical sense (each
optimization removes one of the dominant per-visit costs at high L),
and recall stays flat at 0.9975 across all three because the
optimizations are mathematically equivalent ranking-preserving
changes.

The before/after for the `11091`/`11094` symptom is the cleanest
evidence: the missing IDs `9717` and `7782` went from "not in the
L=200 frontier at all" to "rank 25 / 47 inside `rerank_budget=64`".
That's the single failure case the probe series isolated; it's now
fixed at the level the diagnostic originally identified.

## Risks and follow-ups

### 1. Vacuum repair still uses grouped-PQ (consistency gap)

`plan_vacuum_fill_candidates_for_target` at `routine.rs:1305-1411`
calls `build_grouped_pq_lut_from_persisted` and `grouped_pq_score_f32`
to score candidates when refilling neighbor slots after a node
dies. The Task 29a swap only touched `amrescan`, not vacuum.

Quality consequence: a freshly-vacuumed neighbor slot may hold
candidates that PQ-scored well but binary-sidecar would have
ranked lower — same failure mode as the original Task 29 recall
ceiling, except localized to repaired slots. Worth measuring after
the fix to confirm the practical impact on this corpus.

**Tracked**: Task 29b, Item 1. Wires the same sidecar prefilter
into vacuum-repair candidate scoring, gated on the same
`ec_diskann.prefilter_kind` GUC. Mechanical change — mirrors what
29a did for the scan path.

### 2. Grouped-PQ stays — it's shared infrastructure

Important correction from the original draft of this feedback:
**grouped-PQ is not dead code.** It is load-bearing for `ec_hnsw`
(its primary scan codec) and `ec_ivf`, and remains the
GUC-controlled emergency rollback path for DiskANN scan and
(post-29b) DiskANN vacuum. The "no deprecated names" rule applies
to *unused* names; grouped-PQ is still used.

What 29b does **not** remove:
- `src/quant/grouped_pq.rs` and the `GROUPED_PQ_CENTROIDS` family.
- `src/am/common/training.rs::train_grouped_pq4_model` and
  `derive_grouped_pq4_code`.
- The `PrefilterKind::GroupedPq` enum variant — kept as the
  documented emergency rollback choice.
- `tuple.search_code`, `search_code_len`, `grouped_codebook_head` —
  all still consumed by the grouped-PQ codepath that vacuum (and
  GUC fallback) keeps active.
- `ec_hnsw` / `ec_ivf` references — out of scope entirely.

### 3. The `ec_diskann.prefilter_kind` GUC end-state — keep

Decision (per 29b plan): keep as a real production
emergency-rollback knob. The doc string at
`options.rs:124-127` currently says "Diagnostic override used for
Task 29a A/B measurement"; 29b updates it to reflect production
intent. 29b also adds the missing pgrx test that mirrors the
existing session-list-size override test.

### 4. Build time — Task 29c gate

The 10k-row build took 492 s. The original Task 29 charter
deferred build-time optimization; per current direction, that
deferral is being revisited because shipping with surprise build
latency would set unreasonable expectations.

**Important correction from the original draft**: Task 26 covers
parallel build for **`ec_hnsw` only**. DiskANN has no parallel
build path and Task 26's DSM scaffolding doesn't apply (HNSW
levels and entry-point semantics are HNSW-specific). So there is
no other lane to defer DiskANN build perf to — it needs its own
scope.

**Tracked**: Task 29c. Measurement-first: profile the 6.7× gap
between in-memory replay (73 s, per `11089`) and full ambuild
(492 s), attack the largest fixable contributor, compare against
`ec_hnsw` build (Task 26 packet 666 has reference numbers) and
pgvectorscale DiskANN if installable, then make a defensible
landing decision. Stop condition tied to diminishing returns, not
a fixed target — to avoid setting unreasonable expectations.

### 5. Latency vs HNSW reference

DiskANN is ~50% slower than HNSW at equivalent recall settings on
this corpus. That's the expected disk-first design trade-off, and
recall (+3pp) and storage (-64%) offset it. The framing for outside
review: DiskANN is the right choice when corpus size exceeds
memory budget; HNSW when it doesn't. Implicit in the Task 29
charter; be ready to articulate explicitly.

### 6. Tied popcount tail (theoretical, unmeasured)

Per `review/11095`, 1536-bit Hamming has typical top-K windows of
~115 bits with ±15 bit per-vector noise (5-15 ties per "tight"
bucket). Aggregate sweep doesn't specifically isolate tie-tail
behavior, but 0.997 recall at default `rerank_budget=64` indicates
the tail isn't leaking materially. If a downstream corpus shows
recall plateauing below 0.99 even with `rerank_budget=200`, ties
become the suspect; cheap mitigation is a secondary tie-break via
`tuple.search_code` PQ score (which 29b explicitly keeps available
for exactly this kind of recipe). File the recipe; don't preempt.

## Path to landing

Two follow-up tasks now stand between the branch and merge to main:

### Task 29b — Cleanup and vacuum consistency
Plan: `plan/tasks/29b-diskann-cleanup-and-vacuum-consistency.md`

Scope:
1. Wire the binary sidecar into vacuum-repair candidate scoring
   (mirrors the scan-path swap from 29a).
2. SIMD verification on `hamming_xor_popcount` via `cargo asm`;
   transliterate pgvectorscale's match-arm-per-length unroll only
   if codegen flags a real issue.
3. Update the GUC doc string from "Diagnostic override" to
   production rollback intent. Add the missing
   `pg_test_ec_diskann_prefilter_kind_override_*` test.
4. Code-shape cleanup: extract a shared prefilter-selection helper
   so `amrescan` and vacuum can't drift; fold the
   `peek_next_active`/`pop_next_active` redundancy in `scan.rs`.

Estimated: 3–5 days.

### Task 29c — Build performance
Plan: `plan/tasks/29c-diskann-build-perf.md`

Scope:
- Phase 1: profile the 6.7× gap between 73 s in-memory replay and
  492 s full ambuild on real-10k.
- Phase 2: attack the top 1–2 contributors. Stop at diminishing
  returns.
- Phase 3: reference comparison vs `ec_hnsw` (Task 26 packet 666
  numbers) and pgvectorscale DiskANN (install if feasible per
  Task 29 Phase 1 charter — was deferred earlier, time to land).
- Phase 4 (conditional): scope Task 29d for parallel build only if
  Phase 3 says single-process isn't landable.

Measurement-first; explicit stop condition; no pre-committed
targets. Goal is a defensible landing decision, not a sprint to a
fixed number.

Estimated: 1–3 weeks depending on what Phase 1 finds.

### After 29b and 29c land

5. One-line note in `plan/tasks/29-diskann-initial-tuning.md`
   marking Phase 3 complete with pointers to packets `11096`,
   `11098`, `11099`, plus the 29b/29c outcome packets.
6. Final landing-readiness packet: refresh measurements with the
   29b vacuum-consistent + 29c-tuned head, and propose merge.

That's it. The branch is in good shape. The probe-driven
optimization path the team ran (build quality ruled out → frontier
scoring isolated → binary sidecar wired → latency cleaned up) is
exactly how this kind of work should go, and the final code is
faithful to the architectural recommendations from the comparison
review. The remaining work is making sure the lane ships
consistently (vacuum) and at a defensible cost (build perf)
before merge — not re-litigating the design.
