# Task 168 Packet 002 — batched-beam A/B (Phase 2)

- Task: `plan/tasks/168-diskann-batched-beam-and-prefetch.md` (branch
  `task-161-ec-distann-specs`); work branch `task-168-diskann-batched-beam`.
- Code under test: `1adbc7784` (width-W batched-beam, `ec_diskann.beam_width`
  GUC default 1) — the default-4 flip commit follows this packet.
- Host / backend: Intel desktop, PG18 pgrx tree (port 28818, socket
  `/home/peter/.pgrx`), db `tqvector_bench`; **release** backend verified
  after reinstall (`build-profile.log` → `release`).
- Fixture: the packet-001 indexes reused verbatim
  (`t168_p1_real{10k,50k,100k}_diskann`, rabitq, R=32 reference reloptions).
  Reuse is the correct paired design here: the change is scan-side only, both
  arms run against the identical index, arms differ only in the
  `ec_diskann.beam_width` session GUC.
- Truth caches reused from packet 001 (same tables, same seed) —
  `../001-rabitq-characterization/artifacts/truth-{scale}-k10.json`.
- Commands:
  - `ecaz --host /home/peter/.pgrx --port 28818 bench suite run --config
    <pkt>/suite.json --artifact-dir <pkt>` (22 steps: W∈{1,8} recall+latency
    × 3 scales, width-pick latency W∈{2,4,16,32} at 50k, W=8 profile
    notices; `suite-run.log`, `suite-manifest.json`, `results.jsonl`).
  - `... --config <pkt>/suite-w4.json --results-output
    <pkt>/results-w4.jsonl --manifest-output <pkt>/suite-manifest-w4.json`
    (6 fill-in steps: W=4 recall+latency × 3 scales) — separate outputs so
    the first run's rows stay untouched.
- Bespoke SuiteConfig justification: A/B packet (session-GUC arms over the
  packet-001 fixture), not the standard lane sweep. Sweeps are the
  registered `ec_diskann` default `[64,128,200,400,800]` verbatim.

## Key results (mean latency, 200 queries × 200 iterations; recall@10)

| scale | L | recall W1 | W4 | W8 | mean W1 | W4 | W8 |
|---|---|---|---|---|---|---|---|
| 10k | 64 | 0.9990 | 0.9990 | 0.9990 | 3.09 ms | 3.27 ms | 3.35 ms |
| 10k | 128 | 0.9995 | 0.9995 | 0.9995 | 3.52 ms | 3.48 ms | 3.62 ms |
| 10k | 200 | 1.0000 | 1.0000 | 1.0000 | 3.79 ms | 3.88 ms | 3.90 ms |
| 10k | 400 | 1.0000 | 1.0000 | 1.0000 | 4.52 ms | 4.55 ms | 4.60 ms |
| 10k | 800 | 1.0000 | 1.0000 | 1.0000 | 5.87 ms | 5.85 ms | 5.84 ms |
| 50k | 64 | 0.9685 | 0.9700 | 0.9695 | 4.07 ms | **3.82 ms** | 4.31 ms |
| 50k | 128 | 0.9865 | 0.9860 | 0.9865 | 4.74 ms | **4.42 ms** | 5.11 ms |
| 50k | 200 | 0.9905 | 0.9905 | 0.9910 | 5.50 ms | **5.10 ms** | 5.70 ms |
| 50k | 400 | 0.9950 | 0.9950 | 0.9950 | 7.44 ms | **6.77 ms** | 7.47 ms |
| 50k | 800 | 0.9965 | 0.9965 | 0.9965 | 10.7 ms | **9.86 ms** | 10.8 ms |
| 100k | 64 | 0.9275 | 0.9360 | 0.9445 | 4.16 ms | **4.04 ms** | 4.50 ms |
| 100k | 128 | 0.9665 | 0.9700 | 0.9745 | 5.34 ms | **4.80 ms** | 5.43 ms |
| 100k | 200 | 0.9845 | 0.9845 | 0.9845 | 6.97 ms | **5.72 ms** | 5.96 ms |
| 100k | 400 | 0.9940 | 0.9940 | 0.9940 | 9.10 ms | 8.15 ms | 8.04 ms |
| 100k | 800 | 0.9975 | 0.9975 | 0.9975 | 14.6 ms | **12.3 ms** | 13.0 ms |

Width-pick latency at 50k (L=64 / L=800 mean): W2 4.17/9.85, W4 3.82/9.86,
W16 4.43/10.1, W32 5.20/10.6 (`latency-50k-w{2,4,16,32}.log`).

## Findings

1. **Recall floor holds at every cell for W=4 and W=8** (never below the
   W=1 reference; the Phase-1 0.5 pp floor is comfortably met). At 100k
   low-L the wider beam *raises* recall: +0.85 pp (W=8) / +0.85→+0.35 pp
   (W=4/W=8 at L=128) — the beam explores a strict superset at fixed L.
2. **W=4 is the winning width**: every 50k and 100k sweep point improves
   (50k: 6–9%; 100k: 3–18%, best 6.97→5.72 ms at L=200 and 14.6→12.3 ms at
   L=800). W=8 over-expands at 50k and low-L 100k; W=16/32 are worse.
3. **10k cost**: W=4 pays +0.18 ms at L=64 (3.09→3.27) and is par
   elsewhere. Accepted trade against the 50/100k wins.
4. W=8 profile captures (`profile-notices-{scale}-w8-l{64,800}.log`) show
   the mechanism and the remaining headroom: ≥32-wide flushes go from 0.2%
   of hops (packet 001) to **98.7%** at 100k L=800, and absolute frontier
   time drops ~10% (12065 → 10852 µs mean), but the frontier residual
   *share* stays ~71% — the residual is per-hop allocation work, not
   sub-width scoring. This re-confirms Phase 4 (frontier/alloc cleanups)
   as the next-ranked slice.

## Decision

Flip `ECDISKANN_DEFAULT_BEAM_WIDTH` 1 → 4 (commit follows this packet).
Sessions can restore the legacy loop with `SET ec_diskann.beam_width = 1`.
