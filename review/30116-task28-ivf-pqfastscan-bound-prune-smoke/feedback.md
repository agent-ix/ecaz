---
reviewer: opus (main-conversation)
date: 2026-04-28
head: f47678a2 (with smoke at fa4fea66)
scope: cumulative review of A1–A10 progress at packets 30107–30116, follow-up to feedback at 30106
---

# Task 28 IVF coder-1 progress review (round 2)

This review reads the code and packets landed since 30106. The previous
feedback at `review/30106-task28-ivf-vacuum-replacement-reuse-smoke/feedback.md`
flagged five code issues (F1–F5) and listed A7 and A9 as the hard merge gates.
This round checks what was closed, what advanced, and what is still owed.

## Status of prior feedback items

| ID | Issue | Status at f47678a2 |
|---|---|---|
| F1 | Dead `live_head_block`/`live_tail_block` writes in `vacuum.rs` | **Closed** in `955add3b`. Fields and the per-posting writes are gone; `record_live_posting` now takes only `heap_tid_count`. |
| F2 | `append_ivf_posting_to_list_range` is O(range) per insert | **Acknowledged**. `955add3b` adds a comment at `src/am/ec_ivf/page.rs:1441` explicitly noting the v1 walk and pointing at a free-space sidecar as the scale-up path. No measurement yet, but the contract is now honest. |
| F3 | Empty list ranges reset to INVALID, blocking refill reuse | **Closed in code** by `955add3b`: `run_bulkdelete` now keeps `directory.head_block`/`tail_block` unconditionally (no special INVALID branch on `live_heap_tids == 0`). The 30107 same-distribution smoke is unchanged from 30106 because that fixture wasn't actually exercising the empty-list path on n32/n64; A3's broader convergence is still owed (see below). |
| F4 | A8 reloption naming divergence (`storage_format` vs spec `quantizer`) | **Closed** in `955add3b` by adding a `quantizer` reloption alias that mirrors `storage_format`, with a conflict check when both are set. |
| F5 | A2 acceptance language stronger than evidence | **Closed** by 30109. 1M-row vacuum scale on n8/n32/n64 ran in ~2.0–2.3 s with HWM steady at 430–436 MB. Streaming primitive verified at scale. |

All five prior code findings are addressed. Good cycle.

## Merge-gate status now

| item | status | evidence | remaining |
|---|---|---|---|
| A1 | landed | 30076 | — |
| A2 | **closed** | 30079 (code) + 30109 (1M scale) | none |
| A3 | partial | 30080 + 30103/30105/30106/30107 | nlists=32/64 same-distribution refill still grows; sustained-churn convergence not demonstrated |
| A4 | landed | 30102 | — |
| A5 | landed | 30102 | — |
| A6 | landed | 30077 | — |
| A7 | **landed with positive 100k smoke** | 30110 (live-stop) + 30115 (PQ suffix bound) + 30116 | second-attempt suffix bound got `n128/p48 p50 240.7 → 173.1 ms` at unchanged `recall@10=0.9920`; flat HWM. Still single-point. |
| A8 | landed | 30081 + 30082 + 955add3b alias | — |
| A9 | partial | 30111 + 30112 + 30113 | no `ec_hnsw` matched-shape comparison at 100k; 1M not yet measured; n256 packet (30114) is empty placeholder |
| A10 | partial | 30084 + 30096 + 30097 + 30111 | recall@100 captured for 100k PQ-FastScan only; missing TurboQuant/RaBitQ recall@100 + memory hwm + cold/warm cache at 100k; needs re-run after A7 |

The branch is materially closer to merge-ready. **A3 convergence, A9 head-to-
head + 1M, and A10 closure** are the three open items; A7 is now landed and
positive but needs to be carried into the A10 matrix.

## New code findings

### G1. `running_top` is maintained even for non-PQ-FastScan profiles

`materialize_probe_candidates` (`src/am/ec_ivf/scan.rs:785`) builds a
`CandidateTopK` for any index whose `pre_rerank_candidate_limit` is set, and
pushes into it on every Vacant insert. For TurboQuant and RaBitQ profiles,
`score_ip_from_parts_with_min_bound` ignores the bound and falls through to
the full score, so the heap maintenance is dead work in the per-posting loop.

Fix: only allocate `running_top` when `quantizer.profile` is `PqFastScan`. The
hot loop already has `quantizer` in scope; gate the `Some(top_k)` branches on
that. Cost is small in absolute terms but this is the per-posting hot path
and the bookkeeping is for nothing on the two other profiles.

### G2. Bound-prune threshold lags on duplicate-better updates

The Occupied entry case (`scan.rs:835–841`) updates
`best_by_heap_tid` when a duplicate heap TID arrives with a better score, but
does not push the improvement into `running_top`. 30115 acknowledges this:
"the threshold can lag but cannot become stricter from duplicate heap TIDs."

This is correctness-safe (a more lenient bound only over-keeps, never
over-prunes), but it leaves pruning power on the table when there are many
duplicates (high-`nprobe` runs and multi-list overlap). Two options, in
order of cheapness:

- Push the *new* candidate into `running_top` on the Occupied-better case
  even though the heap may already contain the old worse score for the
  same heap TID. The heap will eventually evict the worse one. Drift in
  retained.len is bounded by duplicate count and resets at the next clear.
- A dedup-aware `running_top` keyed by heap TID — heavier; only worth it
  if a profile shows G1+G2 together account for a measurable share of
  scan time.

I'd defer G2 unless a profile shows it matters; G1 is the cheap one.

### G3. `consume_live_tid_budget` returns `false` for `heap_tid_count == 0`

`scan.rs:863–880` short-circuits `heap_tid_count == 0` as "not consumed →
skip." Today this is unreachable because `posting.deleted` is filtered
above and a non-deleted posting has at least one heap TID, but the comment
is missing and the early return looks like it could leak a bug if either
invariant changes. Either:

- assert `heap_tid_count > 0` and panic instead (matches the actual
  invariant), or
- add a one-liner comment that this is the empty-heap-TIDs guard for a
  posting that survived the deleted filter but has no live TIDs.

Mechanical, but the function reads slightly wrong without it.

### G4. `IvfPreparedQuery::PqFastScan { suffix_max }` adds per-query state that the prior A7 trial (30078) called out as the failure mode

The 30078 packet's lesson was "the next A7 attempt should avoid adding
substantial per-query prepared state in the dev PG path." The 30115
implementation does add per-query state: a `Vec<f32>` of length
`group_count + 1`. Group count for the n128/p48 100k surface is small
(~96 bytes for 24 groups), so the trial-vs-trial difference is the
*shape* of the bound, not the size of the state. The 30116 smoke is
positive at this scale, so the state size is fine here, but flag it for
the larger-dimension/larger-group-count regime: at d=3072 with `group_size=8`
the suffix becomes 384 entries (~1.5 kB), still small but no longer free.

Add a follow-up packet that reruns the A7 smoke at higher dimensions and
larger group counts to confirm the suffix-max state isn't a regression
when group_count grows.

### G5. Empty packet directories under `review/`

`review/30111-task28-ivf-a9-100k-head-to-head/artifacts/` and
`review/30114-task28-ivf-a9-100k-n256-memory-recall100/artifacts/` are
empty with no `request.md`. These are placeholder dirs without artifacts
or a request. Either populate them or remove them — empty packets pollute
the index and confuse cumulative reviews.

## Advice on closing the remaining gate items

The three remaining items are A3, A9, and A10. Close them in that order;
they layer.

### Close A3 (sustained churn convergence)

The 30106/30107 smoke shows nlists=8 reuses fully but n32/n64 grow on the
5k same-distribution fixture. After F3 the empty-list range is preserved,
so the remaining growth is not "lost ranges" — it is one of:

1. **Cross-list page sharing during build.** If multiple lists' postings
   were packed into the same physical block at build time, vacuum can
   compact within a list but the block isn't reachable from another
   list's range. Range-reuse insert on list X never sees the freed slots
   in list Y's blocks.
2. **Directory tuples shared with posting blocks.** Same shape — if a
   directory tuple shares a block with postings, the range walk on a list
   skips that block.
3. **`P_NEW` path winning a race.** If the range walk doesn't find a
   block fast enough (F2 lock contention), inserts fall through to
   `P_NEW`.

The cheapest A3 closure that doesn't require new on-disk metadata:

- **One-shot diagnostic packet.** On the 30106 fixture at n=64 after
  refill, dump per-block: how many postings, per which lists, plus dead
  slots. This will tell you which of (1)/(2)/(3) is the cause. The
  packet is small — `pageinspect` plus a SQL aggregation, or extend
  `debug_ec_ivf_*` helpers in `src/lib.rs` to walk every block and
  report list ownership.
- **If (1) or (2) dominates,** the right v1 fix is build-time list
  segregation: never put two lists' postings on the same block. This
  costs a tiny amount of build-time space (rounding up per list to the
  next page) but gives clean range reuse forever after. It's compatible
  with the existing range-walk insert path.
- **If (3) dominates,** keep the v1 walk, but bias the list directory
  to remember a "free hint" block so the walk starts at a known-empty
  position instead of head. This is less invasive than a free-space
  sidecar.

The acceptance criterion to write into the A3 closure packet:

> Sustained insert+delete+vacuum for N cycles on a 100k-row corpus at
> nlists ∈ {32, 64} grows index size by less than X%, where X is
> documented and chosen against the per-cycle delete fraction.

Pick a concrete N (10 cycles) and X (10% over baseline live-row size at
the steady-state insert count) before running, so the result reads as
pass/fail.

### Close A9 (100k+ head-to-head)

A9 needs three things that are still open:

1. **`ec_hnsw` on the same column at 100k**, with the same query set. The
   30077 matrix did this at 10k; replicate the shape at 100k. Capture
   build time and index size for both, plus recall@10/100 and
   p50/p95/p99 latency at matched recall targets.
2. **1M slice.** Per task wording, A9 is 100k *and* 1M. The chunked
   corpus loader is in place. Run it for `ec_ivf` first; HNSW on 1M is
   slow but the comparison is only useful if both run.
3. **Cold/warm cache state per row.** Today `--sample-backend-memory`
   captures HWM but not whether the buffer pool was cold. Add a cold
   variant: `pg_prewarm` off + `pg_buffercache` snapshot before, then
   the run, then snapshot after. Two rows per condition (cold, warm)
   per profile.

Sequencing: the A9 100k IVF lane is mostly done across 30111/30112/30113;
add the HNSW lane and the cold/warm split, then run 1M. Plan for one
rebuild of each variant at 1M scale; capture build wall time and peak
build memory while you do it (those are A9 deliverables).

The two empty packet dirs (G5) look like they were meant to host this
work. Either fill them or remove them and start fresh.

### Close A10 (head-to-head quantizer recommendation)

A10 needs the quantizer comparison reread *after* A7. The 30097 refresh
captured TurboQuant vs PQ-FastScan g8 at 10k/25k matched-width 750
before A7 landed. Re-run that matrix at the post-A7 head and add:

- **RaBitQ** on the same fixtures. 30084 dropped RaBitQ as
  latency-uncompetitive, but A10 wants three numbers per row, even if
  one of them loses. The honest write-up requirement is in the task
  file.
- **Recall@100** for all three at all corpus sizes. Today only the
  PQ-FastScan 100k path has this (30111).
- **Memory HWM** per variant (the patch in 30112 makes this a flag,
  not new code).
- **Cold/warm cache state** per measurement. Same approach as A9.
- **Build time + index size** per variant per corpus.

The recommendation packet (30096 supersedes earlier ones) should be
rewritten over this matrix. Keep the same posture — recommend
`pq_fastscan, pq_group_size=8` for 100k+ high-dim, keep `auto` =
TurboQuant until a separate task changes the default — but ground it in
the post-A7 numbers, not the pre-A7 ones.

### A practical next-week sequence

1. **G1, G3, G5** — small cleanups, single commit each.
2. **A3 diagnostic packet** — pageinspect/debug-helper output on the
   30106 nlists=64 fixture. Decide between list-segregated build vs
   free-hint cursor based on what the diagnostic shows. Implement and
   re-smoke.
3. **A9 HNSW lane at 100k** — reuses the 30111 IVF surface; add the
   matched-shape `ec_hnsw` index and rerun the same recall+latency
   harness with `--sample-backend-memory`.
4. **A9 1M slice** — IVF first, then HNSW. Build + scan in one packet.
5. **A10 closure matrix** — run all three quantizers at 10k/25k/100k
   with recall@100, HWM, cold/warm. Rewrite the recommendation packet.
6. **G2 / G4 follow-ups** — only if A10 numbers show a high-dim or
   high-duplicate regime where the bound-prune state matters.

After (5), the merge gate is closed. (6) becomes a follow-on task in a
fresh number range, not a merge blocker.

## What is strong about this round

- Five for five on the prior feedback. Including the alias choice on F4
  (additive, conflict-checked) which preserves the more-accurate
  `storage_format` name while satisfying the spec's literal wording.
- The A2 1M scale measurement is exactly the right shape for the gate:
  three nlists points, HWM band measured, no inflated index-size claim.
- The A7 second attempt is grounded: 30078's lesson ("don't add per-
  query prepared state on the dev path") is honored in spirit by keeping
  the suffix-max small and PQ-FastScan-only, and the 30116 smoke is the
  right sanity check before claiming the lever.
- Explain counters added in the same slice (`record_posting_visited`,
  `record_posting_pruned_by_bound`, `record_heap_tids_scored`) make A7
  observable, which will pay back during the A10 matrix when you need
  to argue *why* one variant won.
