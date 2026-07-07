# Manifest — Task 162 packet 002 (M0 bench cells)

- Head SHA of measured build: `1fd015935` (branch `task-162-ec-distann-m0`)
- Task bucket: `reviews/task-162/002-m0-bench-cells/`
- Host: Intel desktop, PG18.3 pgrx tree, port 28818, socket `/home/peter/.pgrx`
- Database: `ec_distann_bench` (fresh, created for this lane; no other lane's
  tables touched)
- Backend build profile: **release**, verified in-run by the precheck step
  (`artifacts/precheck-host.log`: `build_profile = release`) after
  `cargo pgrx install --release` — the debug-`.so` trap was re-armed by the
  packet-001 pg_test runs and explicitly cleared before any latency number.
- Fixtures: staged real corpora `data/staged-current/ec_real_{10k,50k}_*`
  (DBpedia/OpenAI 1536-dim, unit-norm), 200 queries, k=10.
- Isolation: one index per replicated corpus table per arm
  (`m0_{scale}_{arm}_*` prefixes).
- Suite config: `../task-162-m0-suite.json` (bespoke — justification: the
  ec_distann M0 parity/codec/C grid is not in the canonical lane sweep; the
  EC_DISTANN profile itself lands in this task). Runner: `ecaz bench suite`.
- Protocol: recall step precedes latency step per arm (task-168 packet 004
  first-point trap); no OS-evict steps.
- Sweep semantics: the profile sweep drives `ec_distann.top_k` (the D9
  early-exit bar). Run 1 (preserved as
  `results-run1-hop-sweep-inert.jsonl` + `suite-manifest-run1.json`)
  swept `hop_rounds` and measured it inert past convergence — that finding
  drove commit `1fd015935`.

## Commands

- 10k arms: `./target/release/ecaz --host /home/peter/.pgrx --port 28818
  --database ec_distann_bench bench suite run --config
  reviews/task-162/002-m0-bench-cells/task-162-m0-suite.json
  --only-tag setup --only-tag ec_real_10k --continue-on-error`
  (2026-07-07, results in `results-10k.jsonl`, also `results.jsonl`)
- 50k arms: same with `--only-tag ec_real_50k --results-output
  .../results-50k.jsonl` (2026-07-07)

## Artifacts

- `results-10k.jsonl`, `results-50k.jsonl` — normalized result rows (all
  cited numbers trace here)
- `suite-manifest.json` — step statuses (10k: distann_tq load/recall/latency
  failed BY DESIGN, see below; 50k: same three; all other steps succeeded)
- `results-run1-hop-sweep-inert.jsonl`, `suite-manifest-run1.json` — run 1
  (hop_rounds sweep, inert; evidence for the sweep-semantics change)
- per-step logs: `load-*.log`, `recall-*.log`, `latency-*.log`,
  `storage-*.log`, `precheck-host.log`

## Key result lines

Raw vector bytes: 1536×4 = 6144 B/row → 10k raw = 61.4 MB, 50k raw = 307 MB.

### Parity A/B, rabitq codec both sides (FR-075-AC-4; recall@10, p50 warm)

10k — ec_diskann: 0.9990@3.24ms (L=64), 0.9995@3.45ms, 1.0000@3.76ms;
ec_distann rbq: 0.9935@1.67ms (top_k=16), 0.9990@2.33ms, 0.9995@3.82ms,
1.0000@5.52ms.
Matched-recall ratios: 0.9990 → **0.72×**, 0.9995 → **1.11×**,
1.0000 → **1.47×**.

50k — ec_diskann: 0.9700@3.99ms, 0.9860@4.51ms, 0.9905@5.05ms,
0.9950@6.67ms, 0.9965@9.84ms;
ec_distann rbq: 0.9150@2.41ms, 0.9545@3.12ms, 0.9840@4.99ms,
0.9880@6.93ms, 0.9950@13.7ms.
Matched-recall ratios: ~0.986 → **1.1×** (0.9840@4.99 vs 0.9860@4.51),
0.9950 → **2.05×** (13.7 vs 6.67 ms).

### D7 codec comparison (recall@10 ceiling / p50 at the top sweep point)

10k: rbq 1.0000@10.1ms ceiling (0.9990 already at 2.33ms); gpq
0.9905@10.1ms; tq at R=32 **fails to build** (record 25,620 B > page
capacity 8,168 B); tq at R=8 (informational) 0.9320@9.43ms.
50k: rbq 0.9950@13.7ms; gpq 0.9245@11.3ms; tq_r8 0.8710@12.5ms.

### NFR-018 storage (index bytes / raw vector bytes; ec_distann index incl.
head sample + directory + codebooks)

10k: gpq 51.8 MiB → **0.88×**; rbq 110.3 MiB → **1.88×**;
(diskann baseline 4.1 MiB → 0.07×).
50k: gpq 130.8 MiB → **0.43×**; rbq 423.6 MiB → **1.38×**;
tq_r8 423.6 MiB → 1.38× (identical to rbq because both records exceed a
half page → one record per page; page-fill waste ≈23% for rbq);
(diskann 20.6 MiB → 0.067×). All measured formats sit well under the
NFR-018 4.0× budget; the D1 4.0×-class risk materializes only for the
TQ-768B-code format, which does not even fit a page at R=32.

### D3 head_index_cap sensitivity (FR-080-AC-4; 50k, gpq codec)

Top-sweep-point recall@10: C=1024 → 0.9330; C=4096 (default) → 0.9245;
C=16384 → 0.9425. Effect ≲0.02 across a 16× C range at the operating
region; storage delta ≈ C×6.1 KB (106.8 / 130.8 / 226.8 MiB); first-query
head-graph build grows with C (mean-vs-p50 skew; C=16384 first recall
point mean 692 ms vs steady-state p50 ~2-11 ms).

### Operational findings

1. First query per backend pays the FR-080 in-memory head-graph
   construction (~10 s at C=4096, ~2 min at C=16384 on 50k). p50/p95 are
   unaffected; means of first sweep points are inflated. Follow-up
   candidate: persist head adjacency or share the cache (FR-082 epoch
   territory).
2. `distann_tq` arms at default R=32 fail with `tuple payload 25620
   exceeds maximum page capacity 8168` — the concrete ADR-085 D1 outcome.
3. Record-per-page quantization wastes ~23% for the rbq format at
   R=32/dim=1536 (6,612 B record → 1/page).
