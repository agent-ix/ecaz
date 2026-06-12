# Task 105 packet 004 — G4 lane artifacts manifest

- Task bucket: `reviews/task-105/`, packet `004-g4-lane`
- Lane: Graviton 4 — AWS `m8g.2xlarge` (Neoverse V2, sve2-128), profile
  `10k-medium`, us-west-2, db instance `i-0ea9252cbf4564ae9`
- Code under measurement: `main=1345ca603` (Task 105 Phase 1 optimization
  slices merged), release backend (`backend.build_profile=release` in every
  suite manifest; backend sha256 recorded in `day1-smoke.log`)
- Database: `tqvector_bench` (restored from corpus base
  `snap-0e9c7743263e61d70`, real DBpedia embeddings)
- Surfaces: isolated one-index-per-table throughout — every variant has its
  own replicated corpus table `t105_<am>_<quant>_<scale>_corpus` (1M tables
  hold 990k rows from `real_1m_ivf_rabitq1_rerank_corpus`)
- Runner: `ecaz bench suite` (FR-038) via
  `ecaz cloud bench --profile 10k-medium --config
  reviews/task-105/002-full-scale-sweep-configs/artifacts/<config>.json
  --suite <suite> --database tqvector_bench`
- End-state snapshot: `snap-0f546929f70d60fb5` (completed, 400 GiB,
  vol-0b0e0427243257592 — all t105 fixtures at all scales)

## Sweep directories

| dir | suite | config (sha256 in suite-manifest) | S3 run prefix (bucket `ecaz-cloud-10k-medium-2ea74dae`) | steps | window (UTC 2026-06-12) |
|---|---|---|---|---|---|
| `gate-clean/` | task105-g4-100k-dispatch-confirm | t105-g4-100k-rerun.json | t105-g4-100k-confirm-clean/20260612T053900Z | 32/32 | 05:39–05:43 |
| `sweep-10k/` | task105-sweep-10k | t105-sweep-10k.json | t105-g4-10k/20260612T054423Z | 71/71 | 05:44–05:47 |
| `sweep-50k-coldcache-datum/` | task105-sweep-50k | t105-sweep-50k.json | t105-g4-50k-quiet/20260612T072904Z | 71/71 | 07:29–07:40 |
| `sweep-50k-warm/` | task105-sweep-50k | t105-sweep-50k.json | t105-g4-50k-warm/20260612T074213Z | 71/71 | 07:42–07:47 |
| `sweep-1m/20260612T133752Z/` | task105-sweep-1m | t105-sweep-1m.json | t105-g4-1m/20260612T133752Z | 71/71 | 13:37–17:08 |

Superseded S3 runs not packetized: `t105-g4-100k-confirm/20260612T044204Z`
(first gate, invalidated by an orphaned-psql fixture mutation; re-run as
`gate-clean`) and `t105-g4-50k/20260612T054749Z` (pre-quiet-protocol run,
superseded by the coldcache-datum + warm pair). Both remain on the EBS
snapshot.

The 50k pair is deliberate: `sweep-50k-coldcache-datum` ran immediately
after 1M stage-A fixture IO (cold page cache at gp3 baseline) and is kept
as a cold-cache datum; `sweep-50k-warm` is the back-to-back warm re-run and
is the citable 50k row, consistent with the quiet-host protocol.

## sweep-1m rebuild provenance (2026-06-12, addresses feedback 2026-06-12-01)

The originally committed `sweep-1m/20260612T133752Z/` (commit `b5135501c`)
was contaminated: the local `cloud bench` artifact sync left stale Intel
lane files in place for 23 of 30 `latency-*-1m.log` files (byte-identical
to packet 005 copies, some reporting `isa=avx2`). The remote run itself was
sound — per-step execution timestamps in `suite-manifest.json` and all 287
`results.jsonl` rows are genuine G4 measurements (verified distinct from
Intel rows for every flagged step).

The directory was wiped and rebuilt from a fresh, heuristic-free
`aws s3 cp --recursive` of the S3 run prefix (objects uploaded 17:08 UTC in
one batch at run end), keeping exactly the manifest `expected_artifacts`
(71 step logs) plus `suite-manifest.json`, `results.jsonl`,
`suite-run.log`, `suite-config.json` — 75 files. Multi-scale debris that
had accumulated in the shared remote artifact dir (10k/50k/confirm logs,
config copies, truth-cache) was dropped; canonical copies live in their own
sweep dirs and in tracked packet 002 files.

Post-rebuild verification (all gates green):

- 71/71 steps `succeeded`; release backend.
- ISA attribution: `isa=neon` (15 logs) and `isa=scalar` (7 logs) only —
  zero `avx2`/`sve` rows anywhere in the lane (AC2).
- 14/14 recall on/off pairs equal on recall metrics.
- No non-trivial file byte-identical to the Intel lane. Remaining identical
  files are benign by construction: 4 of 13 `storage-*-1m.log` (pure
  deterministic size tables from identical fixture builds) and
  `suite-run.log` (pure command echo of the shared config, no timestamps or
  host content).
- Headline 1M p50 kernel on/off deltas at matched sweep points
  (nprobe/list_size 16|64 or ef 80|160): SPIRE TQ 62.3/80.5 ms (−23%) @16
  and 165.8/230.7 ms (−28%) @64; DiskANN TQ −13% @64, −14% @128; DiskANN
  pqfs-binary −7% @64, −9% @128; remaining HNSW/SPIRE rabitq cells within
  ±2%. IVF on/off pairs read ~0% by construction: the config's off arm
  omits `--ivf-scratch-soa-batch-decode` rather than forcing the GUC off,
  and after the ADR-077 §4 default flip both arms run batch decode — the
  IVF pairs are same-config noise-floor pairs, not a kernel A/B. The IVF
  differential evidence is Task 99's explicit A/B (−44% G4 / −70% Intel
  at 100k); kernel engagement in both arms here is confirmed by counter
  rows (51–71% of query time at 1M on G4).

Note: `results.jsonl` `artifact` fields contain the remote runner's
config-relative paths (`reviews/task-105/002-.../artifacts/...`); the
packet-local copies in this directory are the canonical review artifacts —
map `artifact` basenames into this directory.

Cross-directory contamination scan (same method) over `gate-clean/`,
`sweep-10k/`, `sweep-50k-warm/`, `sweep-50k-coldcache-datum/`: zero bad-ISA
rows, zero Intel-identical non-storage logs (the `sweep-10k/suite-run.log`
identity is the same benign command-echo case).

## Other artifacts

- `day1-smoke.log` — host preflight: ISA probe, backend sha256, focused
  `cargo test --lib` filters (`quant::isa`, `rabitq32`, `lut32`,
  `candidate_batch`) with `--skip pg_test_` (debug-install trap guard).
- `fixtures-10k-50k.log` — staged fixture build log (10k/50k tables +
  indexes, stage-end `vacuumdb --analyze`, quiet-host check). 1M fixtures
  were built in four staged SSM chunks (A=src+IVF, B=SPIRE, C=DiskANN,
  D=HNSW) with disk guards; the 400 GiB live gp3 grow is visible in the
  snapshot size.
