# Task 59 / 004-closeout — Artifact Manifest

- **HEAD SHA at close:** (refresh on commit) — closeout adds 2
  parallel.rs folds (`descriptor_region_ptrs` combination +
  `initialize_parallel_scan_target` thin-wrapper deletion) on top of
  the slice 003 HEAD `9be8cd362`. Combined subsystem lands at
  **33 (-35.3%, target met)**.
- **Branch:** `task-59-parallel-stream-burndown` (~7 commits + this
  closeout commit ahead of `main`).
- **Task bucket / packet path:** `reviews/task-59/004-closeout/`.
- **Lane / fixture / storage format / rerank mode:** HNSW profile
  only (parallel.rs is on the HNSW build_parallel path); 10k + 100k
  corpora from `fixtures/m5_diskann_real{10k,100k}/`; 1536-dim ip
  metric; m∈{8,16} at 10k, m=16 at 100k; ef_construction=128;
  ef_search sweep {40,80,120,200,400} at 10k, {80,120,200,400} at
  100k; rerank n/a (HNSW direct).
- **Shared / isolated surface:** isolated per-corpus per-AM per-m
  prefix (`ec_real_10k_hnsw_m8_idx`, `ec_real_10k_hnsw_m16_idx`,
  `ec_real_100k_hnsw_m16_idx`).

## Artifacts

| File | Source | Command | Timestamp (UTC) | Notes |
| --- | --- | --- | --- | --- |
| `suite.json` (packet root) | hand-written, mirroring `benchmarks/task-50-m5-hnsw-baseline/suite.json` | n/a | 2026-05-25 | 8-step `ecaz bench suite` configuration |
| `pgrx-install.log` | local build | `cargo pgrx install --release --no-default-features --features pg18` | 2026-05-25 | Required so PG18 loads the Task 59 closeout HEAD binary |
| `suite-manifest.json` | `ecaz bench suite run` | suite runner | 2026-05-25 | Per-step status: 8/8 `succeeded` |
| `results.jsonl` | `ecaz bench suite run` | suite runner | 2026-05-25 | Structured per-trial results |
| `suite-run.log` | `ecaz bench suite run --log-file ...` | suite runner | 2026-05-25 | Full suite stdout/stderr |
| `suite-stdout.log` | `ecaz bench suite run ... | tee` | suite runner | 2026-05-25 | Operator-facing summary stream |
| `corpus-load-ec_real_{10k,100k}-hnsw.log` | load step | `ecaz corpus load` | 2026-05-25 | Per-corpus load logs (1536-dim, m=8/16) |
| `recall-ec_real_{10k,100k}-hnsw.log` | recall step | `ecaz bench recall` | 2026-05-25 | Per-corpus recall sweep tables |
| `latency-ec_real_{10k,100k}-hnsw.log` | latency step | `ecaz bench latency` | 2026-05-25 | Per-corpus latency sweep (mean / stddev / p50 / p95 / p99 / max) |
| `storage-ec_real_{10k,100k}-hnsw.log` | storage step | `ecaz bench storage` | 2026-05-25 | Per-index size + bytes/row |
| `src-total-post-003.txt` | `scripts/unsafe_block_count.sh src` | aggregate sum | 2026-05-25 | src/ total post slice 003 = **755** (baseline 771, Δ=-16) |

## Key Result Lines Cited

- parallel.rs: 34 → **20 (-41.2%)** — §Exit target ≤22 exceeded.
  Verified: `grep -c 'unsafe {' src/am/common/parallel.rs == 20`.
- stream.rs: 17 → **13 (-23.5%)** — structural-ceiling claim filed
  in slice 003; reviewer approved on merits at `1f263bc94`.
- **Combined subsystem: 51 → 33 (-35.3%) — task-level target ≤33 met.**
  Verified: 20 + 13 = 33.
- src/ total: 771 → **755 (Δ=-16)** — matches the parallel.rs +
  stream.rs combined delta (no other files touched per scope-lock;
  Task 56.1 was doc-only).
- Bench gate: 8 / 8 `succeeded`. Per-step status in
  `suite-manifest.json`.
- Recall Δ vs M5 baseline: identical at 10k across all 5 ef_search;
  100k Δ within ±0.0014 (well inside the M5 baseline ci95 bands).
- Latency Δ vs M5 baseline: ≤ 0 at every ef_search × corpus point
  (slightly faster on the 100k corpus, likely measurement
  variability; never a regression).
- Storage Δ vs M5 baseline: 0 across all 3 index variants
  (`ec_real_10k_hnsw_m8_idx`, `_m16_idx`, `ec_real_100k_hnsw_m16_idx`).

## Reviewer-driven decisions captured

1. **Slice 002 fix-up (reviewer seq 02 + 04):** "within rounding"
   floor framing rejected → parallel.rs pushed 24 → 22 (-35.3%) via
   two cfg-arm consolidations; 8 missing safety docs added with
   substantive contracts.
2. **Slice 003 (reviewer seq 01):** structural-ceiling claim
   approved on merits because the per-block enumeration + fold-attempt
   analysis distinguishes it from the Task 58 close-below-floor
   pattern; combined-subsystem target miss flagged with Option A /
   Option B disposition request for closeout.
3. **Closeout (this packet):** Option A taken — parallel.rs pushed
   22 → 20 via two more honest folds, absorbing the stream.rs
   structural-ceiling +2 into the combined-subsystem target. **Target
   met at -35.3%.**

## Cross-references

- Slice 001 baseline: `reviews/task-59/001-execution-plan/artifacts/baseline_counts.txt`.
- Slice 002 packet: `reviews/task-59/002-parallel-typed-views/`.
- Slice 003 packet: `reviews/task-59/003-stream-typed-views/`.
- Task 56.1 packet (doc-only follow-up): `reviews/task-56/007-doc-parity-followup/`.
- M5 baseline (bench anchor): `benchmarks/task-50-m5-hnsw-baseline/`.
- Task 50/448 structural-ceiling precedent: `reviews/task-50/448-hnsw-burndown-refreshed-closeout/`.
- Task 56/006 at-floor precedent: `reviews/task-56/006-closeout/`.
- Memory rules honored: see request.md §Cross-references.

## Re-run command

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run --config reviews/task-59/004-closeout/suite.json \
  --log-file reviews/task-59/004-closeout/artifacts/suite-run.log
```
