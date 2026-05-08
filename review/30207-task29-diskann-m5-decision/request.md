# Task 29 ec_diskann M5 Apple-Silicon Decision Packet

Reviewer: top-level summary closing the current ec_diskann
Apple-Silicon optimization round. Fixes a clean stopping point
before chasing measurement-driven follow-ons that need real
infrastructure (cold-cache harness, deeper rerank-loop
restructure, FastScan kernel work).

## Branch state

Branch `ec-diskann-apple-neon-rerank`, ahead of `origin/main` by
8 commits. Three are code checkpoints, four are review packets,
one is a revert.

| commit | role | promotes? |
|---|---|---|
| `dceda05` | NEON exact rerank inner product | yes |
| `35f8539` | packet 30204 (synth smoke) | n/a |
| `eda51e9` | packet 30204 (real-data extension) | n/a |
| `e191a9e1` | heap-TID-ordered rerank fetch | yes |
| `4154fcb6` | packet 30205 (heap-TID order A/B) | n/a |
| `e8c2ad76` | trial: prefetch heap rerank blocks | NO |
| `45557959` | revert prefetch | yes (back to 30205 state) |
| `55d27202` | packet 30206 (prefetch negative result) | n/a |

Current best ec_diskann Apple-Silicon code state: post `e191a9e1`
(NEON kernel + heap-TID-sorted rerank fetch). All recall metrics
are unchanged on every fixture; recall is `1.0000` on the kernel-
stress lane.

## Apple-Silicon levers tried

Same kernel-stress lane on every arm: `m5_diskann_real10k_w800`
(real DBpedia-style 1536d 10k corpus, `rerank_budget=800`,
swept at L=800, 200 iterations / pass, two passes per arm,
warm cache, `--force-index`).

Pass-averaged p50 deltas vs `origin/main` scalar baseline:

| trial | code SHA | p50 vs scalar | recall | promoted? |
|---|---|---:|---:|---|
| 30204 NEON kernel only | `dceda05` | `-1.1 ms` (`-6.7%`) | 1.0000 | yes |
| 30205 NEON + heap-TID order | `e191a9e1` | `-1.8 ms` (`-11.0%`) | 1.0000 | yes |
| 30206 NEON + heap-TID + prefetch | `e8c2ad76` | `-1.6 ms` (`-9.8%`) | 1.0000 | NO (slightly worse than 30205, reverted) |

(Combined `30204 + 30205` p99 vs scalar: `-3.3 ms` (`-17.5%`).
The `30206` row's `p50 vs scalar` is `30205` minus `+0.2 ms` from
the `30206` regression, see that packet for the per-percentile
breakdown.)

The NEON kernel and heap-TID-sorted fetch are the two real
Apple-Silicon-specific wins this round; they stack to roughly
`-1.5-3.5 ms` across `min`/`p50`/`p95`/`p99` on the kernel-stress
lane vs scalar+unordered, with recall held fixed.

The prefetch trial did not promote on this fixture (warm cache
plus a structural double-pin in the synchronous-drain
implementation).

## Levers that turned out not to apply

These were on the candidate list at the start of the round but
ruled out by reading code or by observation; recording so a
future agent does not re-traverse them:

- **`pg_detoast_datum` per rerank row.** `pg_column_size(embedding)`
  on `m5_diskann_real10k_w800_corpus` is `6144` bytes (inline,
  no TOAST), so `pg_detoast_datum` short-circuits and returns
  the original pointer with no allocation. There is no per-row
  detoast cost to optimize on this fixture; the lever is
  conditional on user-controlled column STORAGE settings and
  TOAST-eligible vector sizes, which is out of scope for an
  Apple-Silicon kernel slice.
- **Per-row Vec<f32> allocation in the rerank closure.** The
  rerank path through `routine.rs::with_heap_source_vector` ->
  `ambuild::with_ecvector_datum_slice` already passes a borrowed
  `&[f32]` into the closure; the only `Vec<f32>` materialization
  in the rerank-shaped helpers is `fetch_heap_source_vector`,
  which is used in the insert / forward-neighbor planning path,
  not in the SQL scan rerank loop.
- **Per-rerank scan-state setup (heap relation / snapshot / slot
  / source attnum lookup).** ec_diskann's `ec_diskann_amrescan`
  already resolves these once per rescan and reuses them for the
  whole rerank loop, so the IVF "cache rerank state per scan"
  fix in `c1a761fd` does not have a corresponding gap on the
  diskann side.
- **`vamana_scan_with` per-rerank-call rerank-budget validation,
  greedy-descent reuse, etc.** All checked; no per-row work
  there beyond what is already structurally needed.

## Open follow-ons (deferred, NOT in scope here)

Each of these has a real reason to be measurement-driven and
not a "polish the same path" continuation. None should be tried
as a kernel-style narrow checkpoint until the corresponding
prerequisite measurement justifies it.

1. **Cold-cache rerun of the prefetch trial.** Packet `30206` is
   warm-cache. A cold-cache rerun on a fixture larger than PG
   shared buffers (think real100k or larger) might re-surface a
   real prefetch win. Requires either harness changes (per-query
   cache flush) or a much larger corpus, plus a much longer
   diskann build. Not free.
2. **Async-overlapping prefetch.** The reverted `e8c2ad76` did a
   synchronous read-stream drain before the rerank loop. A
   correct async overlap would hold the read stream open across
   the rerank loop and consume buffers as rerank rows are
   scored. Structurally a bigger change to
   `scan::vamana_scan_with`; should not be tried before a
   cold-cache measurement (item 1) shows it would matter.
3. **NEON FastScan for the binary-sidecar prefilter scorer.**
   `quant::grouped_pq::grouped_pq_score_f32` is the scalar
   reference; the comment explicitly notes "Batched (32-wide)
   FastScan scoring still" deferred. A NEON `vqtbl1q_u8`-based
   FastScan would also need int8 LUT precision, which is a
   recall change, not just a kernel swap. Out of scope for an
   Apple-Silicon round that promised "no recall regression".
4. **Source-decode optimization on TOAST-eligible storage.** If
   a future fixture stores `ecvector` externally (large vectors
   plus narrower rows, or user STORAGE EXTERNAL), the per-rerank
   `pg_detoast_datum` cost stops being free and becomes worth a
   second look. This packet documents only that the current
   fixture does not exercise that case.

## Recommendation

Stop here for this Apple-Silicon round. Leave the branch at
`55d27202` (the prefetch negative-result packet on top of the
heap-TID revert) and let an outside reviewer triage all four
review packets (`30204`, `30205`, `30206`, `30207`) before
either:

- merging the two real wins (`dceda05` + `e191a9e1`) plus their
  packets to `main` and discarding the prefetch trial /
  revert / negative-result packet, or
- merging the entire branch as-is so the negative result is
  preserved in history along with the wins.

That decision belongs with the reviewer / repo owner, not with
this Apple-Silicon slice.

The handoff bar stays satisfied:

- two narrow Apple-specific code checkpoints (NEON kernel,
  heap-TID-sorted fetch),
- both packet-local Apple measurements with two passes per arm
  on a real-data fixture,
- one defensible negative result with structural reasons
  recorded,
- one explicit deferred follow-on list with prerequisites,
- recall preserved at every checkpoint.

## Artifacts

This is a top-level decision summary — there are no new
measurements in this packet. The numbers cited come from the
packet-local artifacts under:

- `review/30204-task29-diskann-m5-neon-rerank/artifacts/`
- `review/30205-task29-diskann-m5-rerank-heap-order/artifacts/`
- `review/30206-task29-diskann-m5-rerank-prefetch/artifacts/`

See each packet's `manifest.md` for SHAs, commands, and
per-pass tables.
