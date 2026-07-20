# Task 163 M1 — TC-039 stitched-vs-monolithic A/B manifest

## Provenance

- **Head SHA (code under test):** `a375d56dd70f364f8c2389201e5524e578f0ff14`
  (branch `task-163-ec-distann-m1`).
- **Build profile:** release — verified `SELECT ecaz_build_profile()` = `release`
  before the run (`artifacts/precheck-host.log`). The `.so` was installed with
  `cargo pgrx install --release --no-default-features --features pg18`.
- **Task bucket / packet:** `reviews/task-163/001-m1-stitch-ab/`.
- **Host:** Intel desktop, PG18 port 28818, socket `/home/peter/.pgrx`,
  database `ec_distann_bench`.
- **Surface:** isolated one-index-per-table (per-arm prefixes `m1_{scale}_{mono,stitch}`),
  `force_index=true`.
- **Corpus:** staged DBpedia real vectors `data/staged-current/ec_real_{10k,50k,100k}`
  (dim 1536; corpus sha256 recorded in each `load-*.log`, e.g. 10k
  `c67c5810…a35e75`). Regenerable via `ecaz corpus`; not committed (NFR-007).
- **Access method / codec:** `ec_distann`, `neighbor_code_format=rabitq` (D7
  default), `graph_degree=32`, `build_list_size=100`, `head_index_cap=4096`
  (all profile defaults). k=10, 200 queries, 200 iterations.

## Command

Bespoke A/B config (variant axis = `build_shards`, the M1 change):

    ./target/release/ecaz --host /home/peter/.pgrx --port 28818 \
      --database ec_distann_bench bench suite run \
      --config reviews/task-163/001-m1-stitch-ab/task-163-m1-stitch-ab-suite.json \
      --artifact-dir reviews/task-163/001-m1-stitch-ab/artifacts

Arms differ **only** in shard count: monolithic = `build_shards=1`; stitched =
`build_shards=4, closure_epsilon=0.1` (the M0 default). Every other option is
the profile default, isolating the stitch effect (A/B-per-change rule). This is
not the canonical lane sweep — reason stated in the suite `description`.

`distinct_recall == recall@10` for `ec_distann`: one global Vamana graph, one
logical record per vector, no partition duplication (ADR-085), so recall@10 is
the distinct_recall the FR-077-AC-1 bar refers to.

## Result: recall@10 A/B (monolithic vs stitched, closure_epsilon=0.1)

Cited from `recall-{scale}-{mono,stitch}.log`. Δ = stitch − mono.

| scale | top_k | mono recall | stitch recall | Δ | mono CI95 | stitch CI95 |
|-------|-------|-------------|---------------|------|-----------|-------------|
| 10k  | 16  | 0.9935 | 0.9950 | +0.0015 | [.9889,.9962] | [.9908,.9973] |
| 10k  | 32  | 0.9990 | 0.9985 | −0.0005 | [.9964,.9997] | [.9956,.9995] |
| 10k  | 64  | 0.9995 | 1.0000 | +0.0005 | — | — |
| 10k  | 100 | 1.0000 | 1.0000 |  0.0000 | — | — |
| 10k  | 200 | 1.0000 | 1.0000 |  0.0000 | — | — |
| 50k  | 16  | 0.9150 | 0.9150 |  0.0000 | — | — |
| 50k  | 32  | 0.9545 | 0.9420 | −0.0125 | [.9445,.9628] | [.9309,.9514] |
| 50k  | 64  | 0.9840 | 0.9810 | −0.0030 | — | — |
| 50k  | 100 | 0.9880 | 0.9860 | −0.0020 | — | — |
| 50k  | 200 | 0.9950 | 0.9930 | −0.0020 | [.9908,.9973] | [.9883,.9958] |
| 100k | 16  | 0.8685 | 0.8080 | −0.0605 | [.8530,.8826] | [.7902,.8247] |
| 100k | 32  | 0.9260 | 0.8750 | −0.0510 | [.9137,.9367] | [.8598,.8888] |
| 100k | 64  | 0.9650 | 0.9390 | −0.0260 | [.9560,.9722] | [.9276,.9487] |
| 100k | 100 | 0.9770 | 0.9720 | −0.0050 | [.9695,.9827] | [.9638,.9784] |
| 100k | 200 | 0.9925 | 0.9885 | −0.0040 | [.9877,.9954] | [.9828,.9923] |

## Finding: FR-077-AC-1 (within 0.001 at 100k) is NOT met at closure_epsilon=0.1

- **10k:** parity holds — Δ within ±0.0015, CI95 bands fully overlap, both arms
  reach 1.0000 by ef=64. Stitch is statistically indistinguishable from
  monolithic and slightly favored at ef=16.
- **50k:** small consistent negative bias (−0.002 to −0.0125). CI95 bands
  overlap at every point, but the point estimates already exceed the 0.001 bar.
- **100k:** clear, CI-separated regression at low ef (ef=16/32 bands are
  **disjoint**), converging to −0.0040 at ef=200. The gap **grows with corpus
  size** (10k≈0 → 50k≈−0.002 → 100k≈−0.004 at ef=200).

Root cause: with a narrow closure band (ε=0.1), boundary nodes that are not in
the overlap band never receive cross-shard edges, so a query whose true nearest
neighbours sit across a shard boundary cannot navigate to them. The boundary
fraction rises with corpus size → the gap scales. Reachability repairs are tiny
(10k: 0, 50k: 14), so this is an edge-*quality* gap, not a connectivity gap.

**Remediation measured in packet `002-m1-closure-sweep`:** widen `closure_epsilon`
to find the ε that closes the gap (recall-vs-ε / build-time-vs-ε tradeoff). The
monolithic path remains the single-node default (`build_shards=1`), so the
program is not blocked (ADR-085 Consequences: a stitch-quality shortfall
degrades build parallelism, not the program).

## Storage & latency: mono ≡ stitch (identical)

The stitch emits exactly one record per vec_id (FR-077-CON-2), so storage is
shard-count-independent. Cited from `storage-*.log` / `latency-*.log`:

| scale | index size (mono = stitch) | latency p50 @ ef=200 (mono / stitch) |
|-------|----------------------------|--------------------------------------|
| 100k | 817.4 MiB | 11.6 ms / 11.2 ms |

Index bytes identical at every scale; p50 latency within noise. The stitch
changes recall and build time only.

## Build time: stitched is faster (parallel shards)

Cited from `load-*.log` (`built m1_{scale}_{arm}_idx in …`):

| scale | monolithic | stitched (4 shards) | speedup |
|-------|-----------|---------------------|---------|
| 10k  | 15.04 s  | 8.95 s   | 1.68× |
| 50k  | 165.08 s | 111.11 s | 1.49× |
| 100k | 386.70 s | 237.72 s | 1.63× |

The parallel per-shard build is the M1 win: ~1.5–1.7× faster index construction.

## Stitch stats (ADR-085 D8 / FR-077-AC-3), closure_epsilon=0.1, shards=4

Captured from the build NOTICE (`ec_distann sharded build: …`):

| scale | duplication_factor | max_shard_size | stitch_peak_union_len | reachability_repairs |
|-------|--------------------|----------------|-----------------------|----------------------|
| 10k  | 1.1794 | 3891  | 88  | 0  |
| 50k  | 1.3226 | 24771 | 110 | 14 |

- **Duplication factor** 1.18–1.32 at ε=0.1 (grows with corpus).
- **Incremental stitch working set** (`stitch_peak_union_len`) 88–110 node ids
  — the largest single-node neighbor union the merge holds at once; the merge
  streams one node group at a time and never holds all unions simultaneously.
- **D8 / FR-077-CON-4 honest accounting (corrected per reviewer 2026-07-07-01):**
  `stitch_peak_union_len` is only the *incremental merge* scratch, NOT the full
  stitch peak. This v1 holds all shard outputs in memory during the stitch
  (it does not spill them to sorted files), so the honest peak-memory figure is
  `shard_output_retained_node_ids` = Σ over shards of (node list + adjacency),
  bounded by `duplication_factor · node_count · (graph_degree+1)`. That is a
  small fraction of the already-resident source-vector set
  (`node_count · dim · 4 B`), which the build holds regardless. The strict D8
  "streamed by vec_id group" bound (spill sorted shard outputs, merge from
  cursors) is a tracked follow-up, not required for M1 correctness. The build
  NOTICE now emits `shard_output_retained_node_ids` alongside the incremental
  peak so both are on record.
- **max_shard_size** 24771/50k shows spherical k-means is quite imbalanced on
  DBpedia (one dominant cluster). This concentrates the boundary and is a
  contributor to the recall gap; the closure sweep (packet 002) tests whether a
  wider band compensates.
- 100k stitch stats land in the packet-002 load logs (same build path).

## Artifacts

- `precheck-host.log` — release-profile + host settings attestation.
- `recall-{10k,50k,100k}-{mono,stitch}.log` — recall@10 sweeps (cited above).
- `latency-{…}.log` — p50/p90/p95/p99 latency sweeps.
- `storage-{…}.log` — index/heap sizes.
- `load-{…}.log` — build wall time + corpus sha256 + stitch NOTICE.
- `suite-manifest.json` — suite runner manifest (step status, arg provenance).
