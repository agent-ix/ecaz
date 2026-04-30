# Task 29 DiskANN — Implementation Review and pgvectorscale Comparison

Branch: `task29-diskann-initial-tuning`
Reviewer: opus-review
Recipient: coder-1

This is a code-level evaluation of `src/am/ec_diskann/` and a comparison to
pgvectorscale's Vamana implementation. It is informed by reading every probe
packet `11087..11094` and the relevant ec_diskann source plus
`~/dev_bak/pgvectorscale/pgvectorscale/src/access_method/` (cloned at HEAD).

The review excludes pgvectorscale's streaming-iterator amgettuple shape per
the request; everything else (graph build, prune, search, distance handling,
node layout) is in scope.


## TL;DR — what the probes actually proved, and where to push

Your probes converged to the right answer. The build is fine; the prefilter
score is the recall ceiling. Specifically:

- **Build distance is already exact source IP** — `ambuild.rs:292-294` passes
  `source_inner_product_distance(source_refs[a], source_refs[b])` to
  `build_vamana_graph`. The Vamana edges are built from the original f32
  source vectors, not the encoded codes. So `11088` (seeded build) and
  `11089/11090` (pass-1 augmentation) couldn't help — they were trying to
  improve a graph that is already as good as the in-memory replay
  (`recall@10 = 0.9995` confirms this).
- **Scan prefilter is grouped-PQ4 over `tuple.search_code` only** —
  `routine.rs:664` is `-grouped_pq_score_f32(&opaque.query_lut, group_count, &tuple.search_code)`.
  At 1536 d with `group_size = min(transform_dim, PQ_FASTSCAN_TARGET_GROUP_SIZE = 16)`,
  the model picks `group_size = 16`, so `group_count = 96` and each node's
  traversal-time fingerprint is **48 bytes** (96 nibbles). That is the source
  of the recall gap. (See `review/11095…/prefilter-detail.md` for the full
  breakdown — pgvectorscale's SBQ at 1 bit/dim is 192 bytes per node, 4×
  larger and bit-level rather than 4-bit-quantized.)
- **The persisted `binary_words` sidecar is dead code on the scan path.**
  Grep confirms the only readers are persistence/vacuum bookkeeping; the
  prefilter closure never touches `tuple.binary_words`. ambuild already
  encodes it, the on-disk tuple already carries it, and at 1536 d that's
  192 bytes per node — which is exactly what pgvectorscale's SBQ uses as
  its full prefilter signal and what gets them ~98% recall.
- **`11094` finally pinned the symptom**: exact IDs `9717` and `7782` never
  enter the grouped-PQ frontier at all. This is now a graph-traversal
  reachability problem under PQ scoring, not a rerank-budget problem and
  not a graph-quality problem. No widening of `rerank_budget` or
  `list_size` will fix this — the missing nodes have never been a
  candidate for rerank.

Ranked next moves (detailed below in §5):

1. **Wire `binary_words` Hamming as the actual prefilter** (or as a first
   stage). Lowest-risk, biggest expected delta. The data is already on
   disk; this is plumbing, not new physical layout.
2. **Replace the linear-scan frontier with a `BinaryHeap` and add the
   pgvectorscale early-stop**. Cheap mechanical change. Won't move recall
   but will move latency a lot — current scan is `O(L · visits)` to find
   the next node to expand, and visits the entire frontier rather than
   stopping when no improvement is possible.
3. **Drop the redundant `read_node` of the picked tuple** in
   `greedy_descent_with`.
4. **Only then** consider tuning grouped-PQ model shape (more groups,
   larger `M`).


## 1. Algorithm-level correctness vs pgvectorscale

### 1.1 RobustPrune

`vamana::robust_prune` (vamana.rs:241) implements the textbook variant: sort
candidates by distance, greedy pick, drop α-dominated tail. One pass at
fixed α. The two-pass driver `build_vamana_graph_with_stats` runs once at
α=1.0, then once at `alpha_final`.

pgvectorscale's `Graph::prune_neighbors` (graph/mod.rs:392) does it
differently in three ways that matter:

1. **Progressive α inside the same pass.** It loops `while alpha <= max_alpha`
   multiplying `alpha *= 1.2` each iteration. It uses a `max_factors[]`
   array per candidate that records the worst α-domination factor seen so
   far. A candidate dropped at α=1.0 may become eligible at α=1.2, etc.
   This is materially different from your two-pass `[1.0, alpha_final]`.
2. **Skips prune entirely if `candidates.len() <= max_neighbors`.** Your
   `robust_prune` is unconditional. For early build the candidate pool is
   often smaller than R and you're paying for nothing.
3. **Symmetric distance state via `dist_state`.** Each pivot loads its own
   `NodeDistanceMeasure` once and reuses it for all later candidates.
   Your `robust_prune` calls `dist(pivot_star.node, v.node)` inside a
   `retain` over the candidate tail — fine for a closure-based design,
   but one consequence is that your build distance closure must be
   genuinely cheap because it gets invoked O(R²) per pivot.

How much this matters for *recall*: your in-memory replay already gets
0.9995, so the build is good enough. The progressive-α pattern matters for
graph diversity at higher α, which would help if you ever increased the
default beyond 1.2. Not a Task-29 priority.

### 1.2 Greedy traversal

This is the recall-relevant comparison.

| | ec_diskann (`scan::greedy_descent_with`) | pgvectorscale (`Graph::greedy_search_iterate`) |
|---|---|---|
| Frontier | `Vec<ScanCandidate>`, linear scan to find min | `BinaryHeap<Reverse<ListSearchNeighbor>>` min-heap |
| Visited set | `HashSet<ItemPointer>` (in `VisitedState`) | sorted `Vec<ListSearchNeighbor>` (insertion via `partition_point`) |
| Pick next | `frontier.iter().filter(!visited).min_by(cmp)` — **O(L) per pick** | `candidates.pop()` — O(log L) |
| Truncate | `frontier.sort()` then `truncate(L)` after every visit — **O(L log L) per visit** | None; visited is unbounded but only first L matter |
| Stop | When no unvisited candidate remains (i.e. visited covers truncated frontier) | When heap top ≥ visited[L-1] (proper Vamana early stop) |
| Tuple read per visit | **2 reads**: prefilter at neighbor-discovery time **and** again at expansion time (line 290) | 1 read; archived view is reused for both score and neighbor list |

Two of these differences are pure waste:

- **The double `read_node` on line 290 of `scan.rs`.** When you pop the
  next candidate to expand, you call `reader.read_node(picked.tid)?`
  to read its neighbor list. But that tuple was already read when the
  candidate was discovered (its score lives in `picked.score`). On a
  100-visit scan that is +100 page lookups and +100 decodes per query.
- **The frontier sort+truncate on every visit.** With L=100 you sort a
  ~100-item Vec every visit. pgvectorscale's heap design means this cost
  is amortized away — push is O(log L), and the natural early-stop means
  you only pop ~1.2L–1.5L times typically.

The third difference — early-stop — affects work, not correctness. Without
the proper Vamana early stop, you keep walking past the convergence point.
Your loop terminates once visited covers the truncated frontier, so it
*does* converge, just later than necessary.

### 1.3 What the linear-scan frontier costs you in numbers

A back-of-envelope: with `list_size=200`, `graph_degree_r=32`, you visit
roughly `L` nodes (≤ 200). Each visit:

- Line 280: linear scan over a 200-entry Vec → 200 ops.
- Line 290: re-read picked tuple → 1 page lookup + tuple decode (~100 ns
  per byte for the search_code path is fine, but the page lookup hits
  the chain page hash).
- Line 299: read each of up to 32 neighbors → 32 page lookups.
- Line 310: sort the 200-entry Vec → ~1500 cmps.

So each visit costs ~33 reads and ~1700 frontier ops. For 200 visits that's
6600 reads and 340k ops. The `BinaryHeap` design saves the 340k by making
each pick O(log 200) ≈ 8 ops.

Your latency table from `11087/11088` shows 84 ms at L=200, 277 ms at L=800
on a 10k corpus. Most of that is page reads, but the linear-scan frontier
becomes a meaningful fraction at L=800 — the per-visit cost goes quadratic
in L while the productive work (page reads, scoring) is linear in L.

### 1.4 The vamana.rs comment on `BinaryHeap`

`vamana.rs:597`:

> Suppress unused-import warning when the module is built without the
> reference test — BinaryHeap is reserved for the optimized greedy search
> variant we'll plug in once profiling shows the linear-scan frontier is
> the bottleneck.

It is. Once you have a recall-acceptable persisted scan, this is worth ~30%
latency at L=200 and more at L=800 with no algorithmic risk.


## 2. Distance & quantization — the actual recall gap

### 2.1 What gets used where

| Stage | ec_diskann | pgvectorscale (SbqSpeedupStorage) |
|---|---|---|
| Build candidate distance | exact f32 source IP (`ambuild.rs:293`) | Hamming popcount on SBQ codes (build distance is approximate too) |
| Build prune distance | same exact f32 source IP via the closure | same SBQ Hamming via `dist_state` |
| Scan prefilter | grouped-PQ4 LUT score over `tuple.search_code` (8 bytes) | Hamming popcount over `tuple.bq_vector` (192 bytes for 1536 d) |
| Scan rerank | exact f32 IP via heap fetch | exact f32 distance via heap fetch |

There are two surprising things here:

**(a) ec_diskann actually uses *better* build distance than pgvectorscale.**
You build with exact source IP; they build with Hamming. Yours produces a
better graph in principle. `11090` confirms this: in-memory Vamana with
exact distance and the same R/L/α reaches `0.9995` recall@10. This is
strictly better than pgvectorscale would do on the same data with SBQ.

**(b) ec_diskann uses a *coarser* and *smaller* scan prefilter than pgvectorscale.**
At 1536 d:

- pgvectorscale SBQ: 1 bit/dim → 1536 bits → **192 bytes per node**.
- ec_diskann grouped-PQ4: `group_size = min(transform_dim, PQ_FASTSCAN_TARGET_GROUP_SIZE = 16)`
  resolves to `group_size = 16` → `group_count = 1536 / 16 = 96` groups →
  96 × 4 bits = **48 bytes per node**.

That's **4× compression** of the prefilter signal vs pgvectorscale, and
the per-bit fidelity is also lower: PQ4 represents each 16-d slice with
one of 16 trained centroids (so each *group* is the same as 4 bits of
information about a 16-d slice, vs SBQ's 16 bits about that slice). The
detailed math is in `prefilter-detail.md`. The recall ceiling at 0.93–0.96
is entirely consistent with this.

`11094` proves it: the missing IDs `9717` and `7782` never enter the
grouped-PQ frontier at all. Other true top-10 IDs make it in but at deep
ranks (28, 31, 48). The PQ score is so noisy at this code size that exact
neighbors get pushed to the tail.

### 2.2 The `binary_words` sidecar — your already-built escape hatch

The on-disk tuple has a `binary_words: Vec<u64>` field (tuple.rs:73). At
build time, `ambuild.rs:259-265` populates it via
`training::derive_persisted_binary_words` whenever
`has_binary_sidecar = sidecar_word_count > 0`. The metadata
`payload_flags` carries `PAYLOAD_FLAG_BINARY_SIDECAR` to record whether
the field is populated. ADR-046 frozen rule 1 already reserved this slot.

But:

- `grep -rn binary_words src/am/ec_diskann/scan*.rs routine.rs` returns only
  bookkeeping (length checks, decode plumbing).
- The prefilter closure at `routine.rs:664` reads `tuple.search_code` only.
- No code path scores `binary_words` against the query.

This is the cheapest possible recall fix: a Hamming popcount over
`binary_words` against an SRHT-rotated query bitvector is the same
arithmetic shape as pgvectorscale's `distance_xor_optimized`. The data
already exists on every node tuple in real-10k indexes built today — you
do not have to rebuild.

The honest version of this fix is: change the prefilter from "score
search_code via grouped-PQ LUT" to "score binary_words via Hamming
popcount of XOR" and treat search_code either as (a) deleted, or (b) a
finer second-stage filter on the candidates that survive. Path (a) is
simpler and matches pgvectorscale parity; path (b) is more work but lets
you keep the grouped-PQ trainer code.

### 2.3 The unit-norm constraint deserves a footnote

`source_inner_product_distance` at `ambuild.rs:322` does
`d = max(0, BIAS - ip)` with `BIAS = 1.0`. This is only well-defined as a
metric-like distance under unit-norm inputs, hence the
`validate_source_vector_unit_norm_sample` machinery in `mod.rs:80` and
the `warn_on_non_unit_source_vector` checks at insert/build time.

pgvectorscale doesn't have this constraint because their build distance
is symmetric Hamming, which is bounded by construction.

This is a footgun if Task 29 ever expands beyond unit-normalized
embeddings. Not a Task-29 blocker but worth noting.


## 3. Structural / API parity items

### 3.1 Streaming vs one-shot scan

You explicitly said to ignore the streaming iterator difference. Just
flagging that the consequence is real for `LIMIT k` queries: the current
ec_diskann path materializes the full `rerank_budget` of results in
`amrescan` (`routine.rs:655-700`), then `amgettuple` is a buffer
walker. So `LIMIT 1` pays the same cost as `LIMIT 100`. Not a recall
issue, just a latency multiplier the user can't escape.

### 3.2 Graph-side data structures

`PersistedGraphReader` (`reader.rs`) does no caching — every `read_node`
goes through the chain page lookup and decode. pgvectorscale wraps reads
in `SbqNode::read` which returns a `ReadableNode` holding an archived
view that lives for the lifetime of the call site. The page is pinned
once and the archived layout is read directly. For a hot scan where the
same neighbor is touched from multiple parents, this is meaningfully
cheaper.

For Task 29 this is below the recall noise — fix it after you've moved
recall.

### 3.3 Tie-breaking

pgvectorscale wraps every distance in `DistanceWithTieBreak`
(`graph/neighbor_with_distance.rs`) which folds the index pointer into
the comparison so that two neighbors at identical quantized distance
sort deterministically by TID. Your `Candidate` does the same in
`vamana.rs:144` (`then_with(|n| node.cmp)`) but `ScanCandidate` in
`scan.rs:94` only ties by block_number then offset_number, both
unsigned u32/u16 — fine for determinism, but no closeness-to-query bias
in the tie.

For PQ at 8 bytes/code, exact ties are common. Whether this matters
depends on how often tied prefilter scores produce arbitrary ordering at
the `rerank_budget` boundary. Worth a quick measurement after the
binary-sidecar wiring.

### 3.4 Start nodes / medoid

You store one medoid TID and fall back to `first_live_tid` if it dies
(`scan::resolve_entry_point`). pgvectorscale stores a `StartNodes`
structure with multiple entry points (mostly for label filtering, but
also avoidable hot-spots). Not relevant to Task 29 since you don't have
labels, and the in-degree max=3250 from `11087` does mean your medoid is
heavily entered. Not a fix to make today.


## 4. Where coder-1's probe trajectory was (and wasn't) right

Reading 11087 → 11094 in order:

- **11087** correctly ruled out persistence corruption and identified
  in-degree hubbing. The latter is a real signal but hubbing is mostly
  a *consequence* of build, not a fixable cause for recall (the in-memory
  replay has the same hubbing and gets 0.9995).
- **11088** tested random seeding. Negative result was correct: random
  seeds don't help when you already have an exact-distance build.
- **11089** "first optimization target: change Vamana build candidate
  generation/pruning" — this turned out to be wrong, but the probe was
  cheap and the pivot afterward was correct.
- **11090** is the most important pivot in the series: confirming
  `0.9995` in-memory recall on the same source vectors moved attention
  off build and onto persisted scan. Good experimental control.
- **11091** SQL-vs-memory diff. Right call. Producing the actual missing
  IDs (`9717`, `7782`) is worth more than aggregate stats.
- **11092 / 11093** rerank_budget sweep. Useful tuning data; correct
  inference that rerank can't recover what's not in the frontier.
- **11094** is the bullseye: confirmed `9717` and `7782` are not in the
  PQ frontier at all. Recommendation to "tune grouped-PQ model shape,
  increase traversal breadth, or hybrid exact/top-up" is the right
  search space, but missed the cheapest move: **the binary sidecar is
  already on disk.**

That's the one big thing the probe series didn't surface: nobody
greppped for `binary_words` in the scan path.


## 5. Recommended next moves, ranked

### 5.1 (P0) Use `binary_words` as the prefilter

**Why first**: data exists on disk for every real-10k index built since
the binary sidecar landed. No rebuild required to A/B test. Brings
prefilter fidelity to pgvectorscale parity (192 bytes vs 8 bytes for
1536 d). Expected to move recall ceiling from ~0.93 to ~0.97+ at the
default `list_size`.

**How**: in `routine.rs:664`, swap the prefilter closure:

```rust
// Before:
|tuple| -grouped_pq_score_f32(&opaque.query_lut, group_count, &tuple.search_code),

// After (sketch):
|tuple| hamming_popcount(&opaque.query_binary_words, &tuple.binary_words) as f32,
```

The query side needs an SRHT-rotated bitvector encoding of the query
matching the sidecar's codec — `training::derive_persisted_binary_words`
on the query bytes after the same SRHT rotation that `encode_query_srht`
already produces. Exists in the trainer; just needs to be reachable from
`amrescan`.

Backfill plan: existing indexes built with `has_binary_sidecar = true`
work as-is. Indexes built without the sidecar need a rebuild — gate the
prefilter switch on `payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR != 0`
and fall back to the current grouped-PQ path otherwise. Keep this one
fallback for compatibility, then once everyone has rebuilt, drop the
grouped-PQ scan path entirely (per the "no deprecated names" rule, do
not let both prefilters coexist long-term).

Validation:

- Rerun `11091` SQL-vs-memory compare with the new prefilter. Expect
  query `10001` to now match exact 10/10 at `list_size ≥ 100`.
- Rerun `11094` grouped-frontier probe to confirm `9717`, `7782` enter
  the frontier early.
- Bench at `list_size = 64, 128, 200` and confirm latency does not
  regress (Hamming over 24 u64 should be SIMD-cheap).

### 5.2 (P1) Replace the linear-scan frontier with a heap

`scan::greedy_descent_with` rewrite:

- Frontier becomes `BinaryHeap<Reverse<ScanCandidate>>` for unvisited
  candidates.
- Visited becomes a sorted `Vec<ScanCandidate>` with `partition_point`
  inserts, like pgvectorscale's `visited`.
- Stop condition: pop only while heap.peek() < visited[list_size - 1].
- Drop the redundant `read_node(picked.tid)` — score and neighbor list
  come from the same read.

This is a recall-neutral, latency-positive change. Worth doing while
you're touching the scan path for 5.1 — same code, same tests.

### 5.3 (P2) Drop the scan-time `read_node` of the picked tuple

If 5.2 happens together, this is folded in. If you want to ship 5.2
later, do this one independently — it's a 5-line diff and saves one
buffer access per visit.

### 5.4 (P3) Only after 5.1, decide grouped-PQ's role

Once `binary_words` is the prefilter, the grouped-PQ codes are doing
nothing useful in the scan path. Two viable endpoints:

- **(a) Delete grouped-PQ from the scan path entirely.** Per the
  "no deprecated names" memory, this is the cleaner end-state.
  `tuple.search_code` becomes vestigial; either remove it from new
  builds or keep it for an experimental finer second-stage filter.
- **(b) Keep grouped-PQ as a per-candidate refinement over the
  Hamming-prefiltered top-K before heap rerank.** This is a real
  optimization (cheaper than heap fetch, finer than Hamming) but only
  worth measuring if Hamming + heap rerank shows a perf gap to close.

Do not pursue (b) speculatively. Wait until the perf numbers from 5.1+5.2
say it would help.

### 5.5 (P4 / optional) Tighten prune to skip when degree ≤ R

`vamana::build_vamana_graph_with_pass1_extra_candidates` always calls
`robust_prune` even when the candidate pool is smaller than `max_degree`.
Mirror pgvectorscale's "skip prune if `candidates.len() <= max_neighbors`"
to cut build time. Estimated savings small (maybe 10–20% of pass-1 cost)
since pass 2 always exceeds R.


## 6. Items that are NOT worth pursuing

- **More pass-1 augmentation experiments.** `11090` proved the in-memory
  graph is already 0.9995. The graph isn't the problem.
- **More `list_size` sweeps with the current prefilter.** `11088`/`11092`
  show recall is flat from L=64 to L=800. Nothing to learn.
- **More `rerank_budget` sweeps.** `11092`/`11093` covered 64→200. The
  ceiling is bounded by what's in the frontier, and `11094` proved the
  missing IDs aren't in the frontier.
- **Refactoring `routine.rs` into smaller modules right now.** It's 3906
  lines and that's a separate cleanup; mixing it with the prefilter swap
  will obscure the recall delta in measurements.


## 7. Summary scorecard vs pgvectorscale

| Aspect | ec_diskann today | pgvectorscale | Gap matters? |
|---|---|---|---|
| Build distance | exact f32 source IP | SBQ Hamming | No — yours is strictly better |
| Build prune | 2-pass [1.0, α_final] | progressive α loop with `max_factors[]` | No for current α=1.2 default |
| Scan prefilter | grouped-PQ4, 48 B/node @ 1536 d, 4 bits/16-dim slice | SBQ Hamming, 192 B/node @ 1536 d, 1 bit/dim | **YES — this is the recall ceiling** |
| Scan rerank | heap fetch + exact IP | heap fetch + exact dist | No |
| Frontier impl | linear-scan Vec | min-heap | Latency, not recall |
| Stop condition | "no unvisited left" | "head ≥ visited[L-1]" | Latency, not recall |
| Tuple reads/visit | 2 | 1 | Latency, not recall |
| Streaming amgettuple | one-shot in amrescan | streaming via iterator | Per-LIMIT latency, not Task 29 |
| `binary_words` sidecar | persisted, unused at scan | n/a | **Latent feature, free recall** |

The recall fix is in row 3 + row 9 — same fix. The latency fixes are
rows 5/6/7. None of them are coupled. Land row-3 first, then the
latency cluster, then revisit grouped-PQ's role.


---

References:
- ec_diskann scan path: `src/am/ec_diskann/scan.rs:255-321`,
  `src/am/ec_diskann/routine.rs:626-705`
- ec_diskann build distance: `src/am/ec_diskann/ambuild.rs:292-294`,
  `src/am/ec_diskann/ambuild.rs:322-334`
- ec_diskann prefilter: `src/am/ec_diskann/routine.rs:664`
- ec_diskann tuple layout incl. binary sidecar:
  `src/am/ec_diskann/tuple.rs:60-115`
- ec_diskann robust_prune: `src/am/ec_diskann/vamana.rs:241-269`
- pgvectorscale greedy: `pgvectorscale/src/access_method/graph/mod.rs:285-385`
- pgvectorscale prune: `pgvectorscale/src/access_method/graph/mod.rs:392-488`
- pgvectorscale SBQ visit: `pgvectorscale/src/access_method/sbq/storage.rs:125-230`
- pgvectorscale streaming scan: `pgvectorscale/src/access_method/scan.rs:209-306`
- Probe series: review packets `11087..11094`
