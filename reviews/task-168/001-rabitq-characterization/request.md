# Review request: Task 168 Phase 1 — rabitq streaming-path characterization

- Branch: `task-168-diskann-batched-beam` (off main `b891c3743`)
- Commits under review: `da3c107db` (flush-width histogram instrumentation)
  + this packet.
- Task: `plan/tasks/168-diskann-batched-beam-and-prefetch.md` (file lives on
  `task-161-ec-distann-specs`).
- Evidence: `artifacts/manifest.md` (source of truth), 18/18-step
  `ecaz bench suite` run on the release backend, rabitq at 10k/50k/100k.

## What Phase 1 establishes

1. **Frontier residual dominates at high L on the rabitq streaming path at
   every scale**: 69.3% / 74.9% / 70.6% of per-query wall time at L=800
   (10k/50k/100k). At the L=64 operating point it is 27.6% / 36.0% / 35.5%.
   This generalizes Task 70's 10k/pq_fastscan finding (77%) to the shipped
   codec and to 50k/100k.
2. **The 32-wide SIMD block almost never fires**: ≥32-wide flushes are
   ≤ 2.4% of hops everywhere; at L=800 the modal per-hop `score_batch` width
   is 1–7 (62.5% of hops at 10k). The one-pop-per-hop loop + dedup starves
   the block kernels exactly as the task predicted.
3. **Graph read+decode crosses the 5% bar only at 100k** (8.5% at L=64,
   12.9% at L=800; ≤ 4.8% elsewhere) — and this was measured warm (recall
   step ran first). Prefetch/cache wins live at 100k+ and colder caches.
4. **Recall floor references** recorded per (scale, L) — later phases must
   stay within 0.5 pp (`artifacts/manifest.md` table).
5. **Exact heap rerank is a flat ~1.9 ms everywhere** (64 heap fetches at
   rerank_budget=64): 67% / 57% / 52% of total at L=64. Out of scope for
   Task 168's four slices (no recall-affecting change), but it is the
   single largest L=64 component — flagged for a follow-up task decision.
   Side observation: the profile NOTICE reports `top_k=64` even though the
   index reloption is `top_k=10` (scan requests rerank_budget-sized
   results); worth a look during Phase 2 review.

## Ranked Phase 2–4 slice list (per the ≥5% gate)

| rank | slice | measured share it attacks | verdict |
|---|---|---|---|
| 1 | Phase 2 batched-beam (width-W) | frontier residual 27.6–74.9% + sub-32 flushes (≥32 fires ≤2.4%) | GO — headline |
| 2 | Phase 4 frontier/alloc cleanups | same residual bucket; sub-counters show it is alloc/move-dominated, not heap/hash ops | GO — re-rank after Phase 2 re-profile |
| 3 | Phase 3 graph-page prefetch + node cache | graph_read_decode 8.5–12.9% at 100k (warm); expect more cold | GO — win expected at 50/100k cold-cache tail |
| — | dedicated prefilter-scoring slice | prefilter_score 1.7–4.6% | SKIP (<5% alone; width fill comes free with Phase 2) |

## Also confirmed in passing

- The standard lane config's diskann load step inherits the stale
  `StorageFormat::DEFAULT = PqFastScan` (`options.rs:66`) because it passes
  no `storage_format` — this packet had to pin `storage_format=rabitq`
  explicitly. Strengthens the task's included default-flip fix.
- rabitq index storage is 431–432 B/row at R=32, scale-invariant.

## Asks

1. Approve the ranked slice list (Phase 2 → 4 → 3 order of expected value;
   execution order stays 2 → 3 → 4 per the task file unless you object).
2. Confirm the recall-floor convention (within 0.5 pp of the packet table
   at each scale/L).
3. Note the rerank observation (#5) — follow-up task or fold into Phase 4
   scope decision is the reviewer's call.
