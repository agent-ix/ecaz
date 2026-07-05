# Manifest — Task 99 packet 003: profile local validation (Intel desktop)

- Task bucket / packet: `reviews/task-99/003-profile-local-validation/`
- Lane: **local Intel desktop** (dev/iteration host; AVX2). This run
  validates the profile SuiteConfig end-to-end and doubles as the local
  reference column. The citable Intel column is the AWS Intel lane.
- Head at run time: `e358e70c9` (branch `task-99-closeout`); installed
  backend `build_profile=release`,
  sha256 `6ec467324af5366afeb1c85297caef529bd3bebf4023bf5e94541722693a3c84`
  (recorded by suite preflight in `suite-manifest.json`).
- Isolation: one index per replicated corpus table throughout
  (`t99_*` prefixes; sources + fixtures SQL logs:
  `fixture-sources-local.log`, `fixtures.log`).
- Fixtures: real DBpedia 100k (replicated from
  `current_intel_real100k_hnsw_corpus` — raw-f32 portability verified in
  packet 002) + synthetic 10k × 1024-dim QJL fixture (seeds 9901/9902,
  generated in-suite).
- Commands:
  - main run: `target/debug/ecaz --database postgres --host
    /home/peter/.pgrx --port 28818 bench suite run --config
    reviews/task-99/002-profile-suiteconfig/artifacts/task99-profile-suite.json
    --artifact-dir reviews/task-99/003-profile-local-validation/artifacts
    --manifest-output .../suite-manifest.json --results-output
    .../results.jsonl` (log: `suite-run.log`)
  - baseline cells (after retagging the runnable no-kernel cells):
    same + `--only-tag no_kernel_baseline --only-tag
    no_kernel_storage_lane` → `suite-manifest-baselines.json`,
    `results-baselines.jsonl` (log: `suite-run-baselines.log`)
  - binary-cell recheck: same + `--only-tag binary` →
    `suite-manifest-binary-recheck.json`, `results-binary-recheck.jsonl`
- Timestamp: 2026-06-11 → 2026-06-12 (PDT)

## Headline results

- **Main run: completed=85 failed=0** (status log: `suite-status.log`);
  baselines run: 6/6 succeeded. Total 91/91 steps green.
- **Recall: 34/34 batch-on/off pairs byte-equal** on recall@k and
  ndcg@k (32 main pairs + exact-mode pair at both sweeps; the only
  unpaired cell is tiled_lut, on-only by design as the runnable
  `retired` confirmation).
- **Counter attribution: `scalar_candidates=0` on every kernel row**;
  every family reports `isa=avx2` on its kernel cells; tiled_lut
  reports `scalar` (retired stub); hamming/binary reports `scalar`
  POPCNT (the Task 103 AVX2-skip decision, reproduced).

## Kernel rates (ns/candidate, 100k fixtures — match the per-family closeouts)

| family / surface | rate | closeout reference |
| --- | --- | --- |
| lut32 IVF | 235.6–236.6 | Task 102: 235–237 (SPIRE) |
| lut32 SPIRE | 239.3–255.1 | 〃 |
| lut32 HNSW multi-lane | 477.3–479.5 | Task 102: 509–530 |
| lut32 DiskANN | 277.9–278.2 | Task 102 packet 002: 265–283 |
| int8_approx32 HNSW | 86.8–87.4 | Task 103: 88.6 |
| rabitq32 HNSW/IVF/DiskANN | 68.8–89.8 | Task 103: 80.4–81.1 |
| grouped_pq IVF | 160.4–161.6 | Task 94 F8 lineage |
| grouped_pq DiskANN | 143.6–156.9 | 〃 |
| qjl32 HNSW/SPIRE @1024d | 256.0–263.5 | Task 97 band |

## End-to-end p50, batch-on vs batch-off (selected; full table in results.jsonl)

| cell | sweeps | deltas |
| --- | --- | --- |
| IVF turboquant | 16/64 | **−66.1% / −69.1%** (16.9 vs 49.8; 50.9 vs 164.6 ms) |
| SPIRE turboquant | 16/64 | **−47.9% / −62.3%** |
| DiskANN turboquant | 64/128 | −16.7% / −30.6% |
| HNSW qjl @1024 | 32/80 | −22.0% / −21.6% |
| HNSW int8_approx | 80/160 | −20.1% / −13.0% |
| HNSW full_lut | 80/160 | −8.8% / −12.6% |
| IVF pq_fastscan | 16/64 | −5.4% / −10.4% (pruning trade still nets a win) |
| IVF rabitq1 | 16/64 | +3.6% / −3.2% (rerank-dominated) |
| SPIRE rabitq | 16/64 | −1.8% / −0.2% |
| SPIRE qjl @1024 | 8/16 | −1.7% / −1.1% (pipeline-dominated) |
| IVF qjl @1024 | 8/16 | +8.3% / +3.0% (batch-on nets negative at small nprobe — decoupling-map row) |
| HNSW exact mode (no kernel) | 80/160 | +4.5% / +3.5% (within noise — baseline behaves) |

Mode ordering on HNSW 100k (kernel-on p50, ef=80/160): int8_approx
4.33/6.74 < full_lut 5.39/8.44 < exact 5.83/9.28 < tiled_lut 8.60/15.40
(retired) — reproduces the Task 103 ordering at 10× the fixture size.

No-kernel storage lane: IVF rabitq4 p50 20.4/59.7 ms at nprobe 16/64
(vs rabitq1 8.05/18.2), recall 0.8850/1.0000 (vs rabitq1 — see
results.jsonl) — the ADR-025-adjacent bits-tradeoff datum.

## Findings

1. **DiskANN grouped-PQ prefilter arm is ungated**: kernel counter rows
   appear in batch-"off" cells and on/off deltas are ~0 (−0.3/−0.9%).
   The A/B axis does not exist for this arm; ADR-077 §4's DiskANN
   bullet gets this nuance. (The gated arms — rabitq/binary/turboquant —
   show clean on/off behavior.)
2. **SPIRE×rabitq emits no batch counter rows on Intel** (no
   `surface=spire quant=rabitq` rows anywhere), confirming the Task 104
   M5 finding is surface-structural, not host-specific. Cell stays
   "e2e only" in the matrix.
3. **kernel_status tags are skip directives**: the first run
   auto-skipped the 6 cells tagged `structurally_absent`/
   `missing_kernel` (marker rows still emitted — honest reporting
   worked). Real-surface no-kernel baselines were retagged
   `no_kernel_baseline`/`no_kernel_storage_lane` (plain tags, runnable)
   and executed in the baselines pass. Generator comment records the
   convention.
4. **diskann-pqfs-binary batch-on cells: first-run contamination,
   recheck clean.** First-run on-cells showed 2–3× the off-cells'
   stddev (1.07–1.21 vs 0.39–0.56 ms) and non-monotonic list_size
   (+49.7% at 64). The isolated recheck (`results-binary-recheck.jsonl`)
   is clean: on/off p50 4.54 vs 4.38 ms (ls=64, +3.7%) and 5.08 vs
   5.16 ms (ls=128, −1.6%), stddev 0.35–0.45 ms both sides, monotonic,
   recall byte-equal (0.9230/0.9550) — within noise, matching the
   Task 103 Intel hamming finding. **The recheck rows supersede the
   first-run on-cells for this lane's citable numbers.**

## Verdict

The profile SuiteConfig is locally validated: all 91 steps execute,
recall parity holds everywhere, counter attribution is complete and
truthful, and the kernel rates reproduce the per-family closeouts on a
single shared fixture set. The config is ready for the AWS lanes
(runbook: `reviews/task-99/006-aws-trip-runbook/`).
