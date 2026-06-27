# Manifest — Task 99 packet 008: Graviton 4 lane (COMPLETE)

- Lane: **Graviton 4 production column** — `10k-medium`, db instance
  `i-00bfd67d8b5ed7959` (m8g.2xlarge, Neoverse V2, **sve2-128**),
  us-west-2, restored from `snap-0e9c7743263e61d70`; `ecaz cloud
  status` cost line ~$0.346/hr.
- Git ref: `63bf4d78c` (main); backend `/usr/lib64/pgsql/ecaz.so`
  sha256 `c785e749…`, `ecaz_build_profile()=release` (suite preflight
  re-recorded per run).
- Database: `tqvector_bench`; sources
  `real_100k_ivf_rabitq1_rerank_{corpus,queries}` (100,000 / 1,000
  rows) → `t99-fixture-sources-aws.sql` + shared `t99-fixtures.sql`
  (11 indexes, `fixtures.log`, rc=0).
- Post-lane snapshot: **`snap-097eb8a8e881384dd`** (corpus base + all
  t99 fixtures at main=63bf4d78c); stack destroyed after snapshot.

## Runs (all suite-driven, FR-038)

1. `profile-run/` — main profile (`task99-profile-suite.json`):
   **91/91 succeeded, 34/34 recall on/off pairs byte-equal,
   `scalar_candidates=0` on kernel rows.** Dispatch attribution:
   lut32→sve2 (all four AMs), grouped-pq→sve2 (gather shape — Task 94
   annotation), qjl32→sve2 blocks + neon octet tails, rabitq32→neon
   (Task 93 routing), int8→neon, hamming→neon, tiled_lut→scalar
   (retired).
2. `neoncap-run/` — `t99-g4-neon-cap-suite.json` (32 steps,
   `ecaz.isa_cap=neon`): **cap held — zero sve/sve2 rows**; every
   kernel cell `isa=neon`; recall unchanged.
3. `task97-run/` — Task 97 qjl32 suite (14/14): `isa=sve2` direct
   counter rows on IVF/SPIRE with the runbook gates; closes the
   Task 97 G4 evidence pending its reviewer.

## Headline: SVE2 loses to NEON on Graviton 4 (equal 128-bit width)

Kernel rates (ns/candidate, averaged across sweeps):

| family/surface | sve2 (default dispatch) | neon (capped) | sve2 penalty |
| --- | ---: | ---: | ---: |
| lut32 IVF / SPIRE | 1,204 / 1,210 | 589 / 597 | ~2.0× |
| lut32 HNSW multi-lane | 3,897 | 1,184 | 3.3× |
| lut32 DiskANN | 1,504 | 710 | 2.1× |
| qjl32 SPIRE (block path) | ~3,000 | 429 | — (mixed cascade) |
| grouped-pq IVF / DiskANN | 144 / 160 | 130 / 119 | 1.1–1.35× |
| rabitq / int8 / hamming | neon-routed in both runs | identical | control cells |

End-to-end p50, default vs neon-capped: **−27% to −45% on every
TQ/lut32 cell** (IVF TQ 41.0→22.9 ms at nprobe=16), −17/−21% SPIRE QJL,
−5% IVF pqfs, worst regression +0.6% (noise). Production
recommendation recorded in ADR-077 §6: prefer NEON over SVE/SVE2 in
`select_highest_isa` on aarch64; SVE2 re-entry only by future
per-family measurement.

## Findings

1. pg_tests compile under plain `cargo test --lib`; pgrx harness then
   attempts a debug extension install. On-host runs must use
   `--skip pg_test_` (`day1-smoke-attempt1.log` → `day1-smoke2.log`).
2. **Stale snapshot catalog**: the restored `tqvector_bench` predates
   `ecaz_build_profile()` and the counter SQL surface. Fixed without
   touching indexes by replaying all 241 ecaz C-function definitions
   from the freshly-created `postgres` DB catalog via
   `pg_get_functiondef` → `CREATE OR REPLACE` (`catalog-refresh.log`,
   0 errors). This is the standard remedy for old-snapshot ×
   new-extension lanes.
3. IVF-QJL cells emit no batch counters (consistent with local Intel
   and the SPIRE-rabitq gap) — e2e + recall evidence only at this
   fixture shape.
