# Manifest — Task 162 packet 003 (Gate G0 kill-check spike, ADR-085 D2)

- Head SHA of measured build: `35f2c4fbd` (extension build `1fd015935`;
  branch `task-162-ec-distann-m0`)
- Task bucket: `reviews/task-162/003-g0-killcheck/`
- Host: Intel desktop, PG18.3, port 28818, database `ec_distann_bench`,
  release backend (same install verified in packet 002's precheck).
- Fixture: `m0_50k_distann_rbq` (50k real corpus, rabitq codec, R=32,
  C=4096) built in packet 002; reused, not rebuilt.
- Suite config: `../task-162-killcheck-suite.json` (bespoke —
  justification: recall-vs-H needs per-step session GUCs pinning
  beam_width/hop_rounds with the top_k exit bar parked at 200; not a
  standard-sweep shape). Runner: `ecaz bench suite`, 28/28 steps
  succeeded.
- Command: `./target/release/ecaz --host /home/peter/.pgrx --port 28818
  --database ec_distann_bench bench suite run --config
  reviews/task-162/003-g0-killcheck/task-162-killcheck-suite.json
  --continue-on-error` (2026-07-07).

## Artifacts

- `results.jsonl`, `suite-manifest.json` — all cited numbers trace here
- `killcheck-recall-*.log`, `latency logs` — per-step output
- `nfr019-counter-sample.log` — one profiled query at BW=32/H=8/top_k=200:
  `rounds_executed=8 records_expanded=256 (== BW x H) neighbors_code_scored=8161
  early_exit=false` — the NFR-019 hard cap observed exactly.

## Measured recall@10-vs-H (50k, rabitq, top_k=200, warm p50)

| BW | H | recall@10 | p50 | p95 |
|---:|---:|---:|---:|---:|
| 4 | 8 | 0.8715 | 2.47 ms | 2.84 ms |
| 4 | 16 | 0.9565 | 4.34 ms | 4.76 ms |
| 4 | 32 | 0.9870 | 7.24 ms | 8.09 ms |
| 4 | 64 | 0.9950 | 13.6 ms | 15.9 ms |
| 32 | 4 | 0.9315 | 6.91 ms | 7.68 ms |
| 32 | 8 | **0.9940** | **12.3 ms** | 13.6 ms |
| 32 | 16 | 0.9965 | 17.4 ms | 24.0 ms |
| 32 | 32/64 | 0.9965 (plateau) | 17.5/18.5 ms | — |

(Full grid incl. H∈{1,2} in `results.jsonl`.)

## Transport input (measured, cited by branch + path per the numbering rule)

Per-round pooled loopback transport cost of the existing SPIRE pipeline,
release build, 4-node local fixture:
`reviews/task-142/016-post-cache-release-ab/artifacts/release-10k-n128-b0/bench-suite/results.jsonl`
on branch `task-142-spire-epoch-cache-overhead` — per-node phase rows:
`candidate_receive elapsed_p50 = 1.000 ms`, `heap_receive elapsed_p50 =
2.000 ms` (p95 3 ms). A distann hop round is one parallel
`ec_distann_expand_nodes` fan-out ≈ one candidate-class round; the D11
heap read happens inside the remote call (node-local), not as a second
round-trip. Projection band used: **1–2 ms per round**.

## Projection: multinode p50 ≈ single-node compute p50 + H × per-round

| Operating point | recall@10 | compute p50 | + H×1 ms | + H×2 ms | vs IVF-100k anchor 37.6 ms |
|---|---:|---:|---:|---:|---|
| BW=32, H=8 | 0.9940 | 12.3 ms | 20.3 ms | 28.3 ms | **under, 1.3–1.9× headroom** |
| BW=32, H=16 | 0.9965 | 17.4 ms | 33.4 ms | 49.4 ms | borderline at the pessimistic bound |
| BW=4, H=64 | 0.9950 | 13.6 ms | 77.6 ms | 141.6 ms | far over — narrow-beam multinode is not viable |

Anchor source: `reviews/task-146/006-anchor-results/` on branch
`task-146-spire-honest-pareto-confirmation` (IVF 100k 0.9980 @ 37.6 ms
p50, release).

## Caveats recorded with the projection

1. Curves are 50k; the anchor is 100k. Per-query work is
   corpus-size-independent by construction (BW×H cap), but the H needed
   for a given recall may grow with corpus size; M2 measures directly.
2. The transport band is SPIRE's payload shape; distann expansion
   responses carry ~R×(8 B + 4 B) neighbor arrays per record (≈ 12 KB per
   32-record batch) — same order as SPIRE candidate payloads.
3. Single-node compute p50 includes all expansion work serially; multinode
   parallelizes it across owning nodes, so the compute term is
   conservative (upper bound).
4. D4 watch item: at BW=32/H=8 the projected transport share is 8–16 ms of
   20–28 ms ≈ 40–57% of multinode p50 — sitting at the D4 baton-passing
   reopen trigger (≥50%). M2's measured hop RTT decides.
