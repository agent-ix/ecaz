# Task 102 Packet 002 Artifact Manifest

- head SHA (code under review): `2f99971c5` (`Route DiskANN TurboQuant
  prefilter batches through the lut32 kernels`), on top of `90a221a20`
  (`Port lut32 NEON kernel to shuffle-repack; add SVE live-width tails`)
- task bucket: `reviews/task-102/`
- packet path: `reviews/task-102/002-neon-repack-diskann-tq-batching/`
- timestamp: 2026-06-10
- lane: local PG18 / Intel AVX2 / TurboQuant no-QJL 4-bit, dim 1536
- fixture / storage: synthetic 2k × 1536d corpus, 64 queries, DiskANN
  `storage_format=turboquant` (`task102_diskann_tq_2k`, default graph
  reloptions), generated and loaded by the suite itself
- surface isolation: own one-index-per-table prefix, created by this packet
- rerank mode: fixture defaults; sweep `list_size=64,128`
- AWS / Graviton: not run; the NEON/SVE changes in `90a221a20` are
  compile-gated off x86 and validate on the G4 pass

## Backend provenance

Run after the focused-test phase deliberately, because the broad
`cargo test --lib diskann` filter executed `#[pg_test]`s and installed a
debug backend (the known trap from
`reviews/task-94/027-latency-width-rerun/feedback/`):

- `install-ecaz-pg18.log`: `ecaz dev install ecaz-pg-test --pg 18`, backend
  SHA `3df91dd8733bb4b4d1fbbf8b08b0cab36b3dc6f1e6d5e57c919879cd280f80eb`
- `restart-pg18.log`: `ecaz dev scratch restart --pg 18`
- `build-profile-probe.log`: `SELECT ecaz_build_profile()` → `release`
- `suite-manifest.json` preflight: `backend.build_profile=release`, same SHA

## Suite

Config: `task102-diskann-tq-suite.json` (7 steps: 2× generate, 1× load,
recall on/off, latency on/off with `--task87-candidate-batch-counters`).
Pre-run `bench suite audit` fails on the not-yet-generated fixture TSVs
(`suite-audit.log`, expected — same shape as the Task 101 packet 002
fixture-generating suite); post-run audit passes (`suite-audit-post.log`).

- `suite-run.log` / `suite-manifest.json` / `results.jsonl` /
  `results-report.jsonl` / `suite-report.log`
- `suite-status.log`: `completed=7 failed=0 skipped=0 dry_run=0
  missing_artifacts=0 stale=0`
- fixtures: `task102_diskann_tq_2k_corpus.tsv`, `task102_diskann_tq_2k_queries.tsv`,
  `load-diskann-tq-2k.log`, `truth-cache/`

## Key result lines (`results.jsonl`)

### Direct counter rows — the missing pairing now exists

| Cell | rows |
| --- | --- |
| latency kernel-on `list_size=64` | `surface=diskann quant=turboquant isa=avx2 kernel_candidates=259589 scalar_candidates=0`, 265.2 ns/candidate, widths `w<8=117 w8-15=4025 w16-31=8918 w>=32=221` |
| latency kernel-on `list_size=128` | `isa=avx2 kernel_candidates=347977` at 282.5 ns/candidate, plus 128 single-lane flushes correctly attributed `isa=scalar` (1368.6 ns/candidate, the scalar fast path) |

The 265–283 ns/candidate kernel rate matches the packet 001 lut32 kernel
ladder (SPIRE full blocks measured 235 ns/c; DiskANN's flushes are
mid-width 8–31, so octet padding adds the expected overhead).

### Recall — byte-equal

`recall@k=0.7516`, `ndcg@k=0.9726` identical for kernel-on and kernel-off
at `list_size=64`.

### End-to-end latency p50

| Sweep | kernel-off | kernel-on | delta |
| --- | ---: | ---: | ---: |
| list_size=64 | 5.44 ms | 4.01 ms | **−26.3%** |
| list_size=128 | 6.37 ms | 4.47 ms | **−29.8%** |

## Focused test logs

- `cargo-test-diskann-tq-counters.log`: the new
  `diskann_turboquant_prepared_prefilter_batch_scores_and_records_counters`
  (1 passed) — batch scores bit-exact with the per-candidate prefilter
  path, polarity included; 39/0 kernel attribution on this AVX2 host.
- `cargo-test-lut32.log`: 11 passed (NEON/SVE entries return None on x86).
- `cargo-test-candidate-batch.log`: 19 passed.
- clippy `--all-targets --no-default-features --features pg18 -D warnings`
  clean at head (not logged; rerun is cheap).
- Known flake, pre-existing: `scan_profile_notice_guc_defaults_to_off`
  panics on pgrx GUC FFI when the broad `--lib diskann` filter runs
  `#[pg_test]`s in parallel threads; it passes alone.
