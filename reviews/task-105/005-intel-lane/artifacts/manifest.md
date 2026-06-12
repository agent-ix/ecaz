# Task 105 packet 005 — Intel lane artifacts manifest

- Task bucket: `reviews/task-105/`, packet `005-intel-lane`
- Lane: Intel — AWS `m7i.2xlarge` (Sapphire Rapids), profile `10k-intel`,
  us-west-2
- Code under measurement: `main=1345ca603` (Task 105 Phase 1 optimization
  slices merged), release backend (`backend.build_profile=release` in every
  suite manifest; backend sha256 recorded in `day1-smoke.log`)
- Database: `tqvector_bench` (restored from corpus base
  `snap-0e9c7743263e61d70`, real DBpedia embeddings)
- Surfaces: isolated one-index-per-table throughout — every variant has its
  own replicated corpus table `t105_<am>_<quant>_<scale>_corpus` (1M tables
  hold 990k rows from `real_1m_ivf_rabitq1_rerank_corpus`)
- Runner: `ecaz bench suite` (FR-038) via
  `ecaz cloud bench --profile 10k-intel --config
  reviews/task-105/002-full-scale-sweep-configs/artifacts/<config>.json
  --suite <suite> --database tqvector_bench`
- Lane end state: stack destroyed 2026-06-12 after the 1M sweep (standing
  operator instruction). End-state snapshot `snap-0338adc6455257604`
  (completed — all t105 fixtures at all scales). The S3 bucket was emptied
  during teardown, so the packet-local copies here are the canonical
  artifacts; the instance-side copies survive on the snapshot.

## Sweep directories

| dir | suite | config (sha256 in suite-manifest) | steps | window (UTC 2026-06-12) |
|---|---|---|---|---|
| `sweep-10k-clean/` | task105-sweep-10k | t105-sweep-10k.json | 71/71 | 06:19–06:22 |
| `sweep-50k-quiet/` | task105-sweep-50k | t105-sweep-50k.json | 71/71 | 06:38–06:42 |
| `sweep-1m/` | task105-sweep-1m | t105-sweep-1m.json | 71/71 | 12:24–16:14 |

The 100k optimization-confirmation evidence for this host is the Task 99
Intel profile run (`reviews/task-99/`, Intel lane; clean IVF TQ
−70.3/−68.9% deltas) — the confirmation gate the operator required before
1M work; it was not re-run under Task 105.

`sweep-10k-clean` and `sweep-50k-quiet` are the verified-quiet re-runs:
the first 10k/50k Intel runs were invalidated by an orphaned-psql fixture
mutation and a post-load autovacuum storm respectively, and were re-run
after the quiet-host check (load < 1, zero vacuum workers,
`pg_stat_progress_vacuum` empty) passed. Only the clean/quiet runs are
packetized.

## Lane verification (all green)

- 71/71 steps `succeeded` at every scale; release backend.
- ISA attribution: `isa=avx2` and `isa=scalar` only across the lane — no
  foreign-lane rows.
- 14/14 recall on/off pairs equal on recall metrics at 1M.
- Headline 1M p50 kernel on/off deltas at matched sweep points: SPIRE TQ
  38.7/61.7 ms (−37%) @16 and 98.2/187.8 ms (−48%) @64; DiskANN TQ −11%
  @64, −5% @128; HNSW rabitq and HNSW TQ full_lut −19%/−21% @ef=160 (flat
  @80). IVF on/off pairs read ~0% by construction: the config's off arm
  omits `--ivf-scratch-soa-batch-decode` rather than forcing the GUC off,
  and after the ADR-077 §4 default flip both arms run batch decode — the
  IVF pairs are same-config noise-floor pairs, not a kernel A/B. The IVF
  differential evidence is Task 99's explicit A/B (−70.3/−68.9% Intel at
  100k); kernel engagement in both arms here is confirmed by counter rows
  (35–45% of query time at 1M).
- Known anomaly for Phase 3: `diskann-pqfs-binary` on/off @list_size=64 is
  +26% (4.84 vs 3.83 ms) while @128 it is −7%; single noisy point
  (stddev ≈ 1.4 ms on ~4 ms p50s), flagged rather than concluded.

Note: `results.jsonl` `artifact` fields contain the remote runner's
config-relative paths (`reviews/task-105/002-.../artifacts/...`); the
packet-local copies in this directory are the canonical review artifacts.

## Other artifacts

- `day1-smoke.log` — host preflight: backend sha256, focused
  `cargo test --lib` filters with `--skip pg_test_` (debug-install trap
  guard).
- `fixtures-10k-50k.log`, `fixtures-1m-stage-a.log` — staged fixture build
  logs (per-AM staged 1M chunks with disk guards; 400 GiB live gp3 grow;
  stage-end `vacuumdb --analyze` + quiet-host check).
