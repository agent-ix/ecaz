# Task 99 item 9 — index × quant × mode profile: design + lane plan

## Purpose

One suite-driven profile of all indexes × quants × modes on shared
fixtures, executed identically on three lanes — local Intel desktop
(validation + reference), **Graviton 4** (pinned 2026-06-10), and
**AWS Intel** (pinned 2026-06-10) — feeding: the ADR-025 TQ-mode
reevaluation, the per-ISA Graviton-vs-Intel price/performance comparison
(AC 4), the IVF batch-decode GUC default decision (pre-closeout review
F4), and the scoring-share vs end-to-end decoupling map (Phase 3).

## Design decisions

1. **Scale: real DBpedia 100k, one scale.** Large enough that the
   working set (~600 MB f32 + codes) exceeds cache and the ADR-025 L1D
   argument is exercised; small enough that ~11 index builds per lane
   are affordable. 1M is deliberately excluded: the May-2026 1M IVF
   rabitq evidence stands (snapshot description), and 1M HNSW/DiskANN
   builds would dominate the trip for no decision-relevant gain.
2. **Dimension coverage:** synthetic 10k × 1024-dim fixture (seeds
   9901/9902) exercises the QJL lanes on HNSW/IVF/SPIRE — the no-QJL
   lane only exists at 1536 (Task 96/97 surface inventory). Generated
   in-suite so every lane builds the identical fixture.
3. **Fixture bootstrap = table replication.** `embedding` is raw f32 in
   the ecvector container, verified byte-identical across AM-profile
   fixtures (6144 bytes = 1536×4; equality-checked across the
   task87_phase6 hnsw/ivf/spire tables). So one source corpus table
   seeds every per-variant table on any lane — including the AWS lanes,
   where the corpus arrives as the `snap-0e9c7743263e61d70` restore
   (IVF-profile tables, same raw embeddings). Per-lane sources file +
   shared `t99-fixtures.sql`; one index per replicated table
   (index-isolation rule). Index shapes mirror the established per-AM
   conventions (see SQL header).
4. **Batch on/off at every cell** where the AM exposes the toggle —
   including the IVF cells, where batch-on trades away suffix-max
   pruning (Task 94 F1): `ec_{hnsw,spire,diskann}.candidate_batch_scoring`,
   IVF via the suite's `ivf_scratch_soa_batch_decode` field.
5. **Absent cells marked, not skipped** (Task 92 markers, incl. the
   Task-104 runnable `retired`): tiled_lut runs batch-on only, tagged
   `kernel_status=retired`; HNSW exact mode tagged
   `kernel_status=structurally_absent` (the no-kernel f32 baseline
   mode); IVF rabitq bits=4 tagged `kernel_status=missing_kernel`
   (real storage lane, no kernel by the Task 93 bits=1 scope decision).
6. **Counters on every latency step** (`task87_candidate_batch_counters`)
   for ISA attribution (`isa=sve2|avx2` expected per lane,
   `scalar_candidates=0` on kernel rows, width histograms).

## Cell matrix (45 bench cells → 84 bench steps + 14 storage + 5 fixture)

| AM | fixture | cells |
| --- | --- | --- |
| HNSW | t99_hnsw_tq_100k | exact-mode isolation (prefilter off): full_lut on/off, int8_approx on/off, exact on/off (structurally_absent), tiled_lut on (retired); default production path on/off |
| HNSW | t99_hnsw_rabitq_100k | bits-1 sidecar on/off |
| HNSW | t99_qjl_hnsw_1024 | QJL on/off (ef 32/80) |
| IVF | t99_ivf_tq_100k | SoA-decode on/off (nprobe 16/64) |
| IVF | t99_ivf_rabitq1_100k | on/off |
| IVF | t99_ivf_rabitq4_100k | on only (missing_kernel) |
| IVF | t99_ivf_pqfs_100k | on/off (pruning-trade axis) |
| IVF | t99_qjl_ivf_1024 | QJL on/off (nprobe 8/16) |
| SPIRE | t99_spire_tq_100k | on/off |
| SPIRE | t99_spire_rabitq_100k | on/off (`attribution_check` — M5 found counters not batch-attributed on this surface) |
| SPIRE | t99_qjl_spire_1024 | QJL on/off |
| DiskANN | t99_diskann_pqfs_100k | prefilter=binary_sidecar on/off (hamming lane) + prefilter=grouped_pq on/off |
| DiskANN | t99_diskann_rabitq_100k | on/off |
| DiskANN | t99_diskann_tq_100k | on/off |

Documented non-cells (not run): SPIRE pq_fastscan (structurally absent —
product gap, Task 104); HNSW grouped-PQ (per-candidate traversal + M5
coverage gap; no production claim to measure); TQ 2-bit (no surface,
Task 96); f32 raw (canonical no-kernel cell); DiskANN QJL (TQ storage is
1536-only on DiskANN).

## Files

- `gen_t99_profile.py` — generator (source of truth for the matrix)
- `task99-profile-suite.json` — generated SuiteConfig (91 steps)
- `t99-fixture-sources-local.sql` — local lane source mapping
- `t99-fixtures.sql` — shared per-variant fixture creation
- `suite-dry-run.log`, `dry-run-manifest.json` — schema validation

## Run sequence (every lane)

1. Release backend: `ecaz dev install ecaz-pg-test --pg 18` → restart →
   `ecaz_build_profile()` = `release` (suite preflight re-records SHA).
   No cargo test between install and bench (pg_test debug-install trap).
2. Lane sources SQL → shared `t99-fixtures.sql` (logs into the run
   packet's artifacts).
3. `ecaz --database <db> --host <socket> --port <port> bench suite run
   --config .../task99-profile-suite.json --artifact-dir
   reviews/task-99/<run-packet>/artifacts --manifest-output
   .../suite-manifest.json --results-output .../results.jsonl`
4. `bench suite audit` + `report` against the produced manifest.

## AWS trip plan (after local validation + packet review)

Single trip, both lanes from the same snapshot restore:

| lane | profile | instance | est. $/hr |
| --- | --- | --- | --- |
| Graviton 4 | `10k-medium` | m8g.2xlarge (8 vCPU, Neoverse V2/SVE2) | ~$0.35 |
| Intel | `10k-intel` | m7i.2xlarge (8 vCPU, AVX2) | ~$0.49 |

Per lane: `ecaz cloud up --profile <p> --from-snapshot
snap-0e9c7743263e61d70 --git-ref <final main>` → `cloud install` →
G4 day-one smoke set (per-ISA parity tests incl. `Isa::Sve2` assertion +
measured vector length, expected `sve2-128`) → lane sources SQL (against
the snapshot's restored corpus tables) → fixtures SQL (~11 × 100k index
builds, the dominant cost — est. 2–4 h) → suite run (est. 2–3 h) →
Task 97 runbook cells (packet 022; rides the same G4 instance) →
`cloud snapshot` → `cloud down --yes`.

**Task 94's deferred G4 pass rides this trip** (its only remaining item
per the packet 028 reviewer verdict; no separate runbook exists — the
task file's old "packet 027" runbook pointer was stale and is fixed).
Closing evidence = this profile's grouped-PQ cells: IVF pq_fastscan
batch-on/off (nprobe 16/64) and DiskANN prefilter_kind=grouped_pq
batch-on/off (list_size 64/128) at 100k with counters, `isa=sve2` rows
and measured vector length, **annotated as the gather-shape SVE2
kernel** if the SVE repack remains deferred (Task 94 reopened-scope
rule). If Task 94's reviewer requires the full packet-025 matrix shape,
supplemental 10k/25k IVF pq_fastscan replicas are cheap to add
on-instance from the same source tables.

**G4 NEON-capped pass (added 2026-06-11, operator decision).** After the
main profile run on G4, run `t99-g4-neon-cap-suite.json`
(`reviews/task-99/004-isa-cap-dispatch/`, 32 steps derived from the main
profile's kernel-on cells) with the new `ecaz.isa_cap=neon` session GUC.
Graviton 4's SVE2 is 128-bit — the same vector width as NEON — so this
pass measures whether preferring SVE2 over NEON in `select_highest_isa`
is actually right per family, instead of assuming it. Expected counter
attribution on these cells is `isa=neon`; that is the cap working.
Recall must stay byte-equal vs the same fixture's uncapped cells for
bit-exact families.

Estimated cost: roughly 6–9 instance-hours per lane → **~$5–8 total**,
plus EBS snapshot retention. Instance types and on-demand pricing are
recorded in the run-packet manifests per the pinned requirement.

The G4 lane additionally closes: Task 97 (approval-gated on G4), the
rabitq32 SVE-vs-NEON-routing decision (IVF ~99% block-coverage datum),
and the grouped-PQ SVE gather-vs-repack annotation.
