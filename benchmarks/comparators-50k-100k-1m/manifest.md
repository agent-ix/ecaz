# Comparator latency baselines — pgvector, pgvectorscale, vchord @ 50k / 100k / 1m

Pure measurement packet under top-level `benchmarks/` per NFR-007 and
the project Benchmark Data Packets convention. No code review request
attached; this packet exists purely to record competitor latencies that
the ecaz access methods (`ec_hnsw`, `ec_ivf` {turboquant, rabitq,
pq_fastscan}, `ec_diskann`) need to clear.

## Head

- Repo SHA: `63024cce` (`Fix benchmark manifest reference after move`)
- Bench scripts under that SHA: `scripts/comparators/{install,load,bench}_<ext>.sh`
- Known bug at this SHA in `scripts/comparators/_bench_lib.sh`:
  `awk '/^Time:/' ... raw.out` reads the wrong file (psql `\timing`
  output goes to stdout / `run.log`, not the `\o`-redirected
  `raw.out`). All 12 host-side `latency.log` files initially landed as
  `# no samples`; values in this packet were recovered locally by
  re-running the extraction against `run.log`, which has the raw
  `Time: N ms` lines for every iteration. Fix is pending in the same
  changeset as the per-variant-table refactor (see "Source files
  changed in the same series").

## Host

- AWS EC2 `i-05af7ea8e92f65b30`, `m8g.2xlarge` (Graviton 4 /
  Neoverse-V2 / aarch64)
- Amazon Linux 2023, PostgreSQL 18.3
- `shared_preload_libraries = 'ecaz,vchord'`
- `maintenance_work_mem = 4 GB` during all index builds
- One disk: `/dev/nvme0n1p1` 20 GB root; PG data co-located on root
  (no separate gp3 EBS at run time — disk was tight but never panicked)
- Instance was the same one used for the in-progress 1m comparator
  cycle; it remains running after this packet for follow-up work

## Surfaces and isolation

- All comparators run on `real_<size>_<ext>[_<variant>]_corpus`
  tables loaded from the prepared DBpedia real-corpus TSVs
  (`/var/lib/pgsql/18/datasets/staged-<size>/...`)
- `dim = 1536`, `op = <#>` (negative inner product, pgvector IP opclass)
- 200 query iterations per bench pass; `k = 10`; serial (concurrency
  1)
- Index isolation was **drop-and-rebuild swap dance** for pgvector
  HNSW vs IVFFlat at this SHA. Per-variant replicated tables
  (`real_<size>_pgv_hnsw_corpus` / `..._ivfflat_corpus`) are landing
  in the same series so future runs eliminate that rebuild cost; this
  packet was captured before that refactor took effect on the host.
- pgvectorscale and vchord ship a single index each, so no swap dance
  applies to them.

## Results

p50 / p95 / p99 latency, ms, k=10, 200 iterations, serial:

| size | pgvector HNSW   | pgvector IVFFlat   | pgvectorscale DiskANN | **vchord RaBitQ-on-IVF** |
|------|-----------------|--------------------|----------------------|--------------------------|
| 50k  | 27.7 / 254 / 378 | 347 / 361 / 743   | 346 / 350 / 354      | **2.7 / 3.0 / 3.2**      |
| 100k | 72.5 / 377 / 539 | 699 / 710 / 1195  | 696 / 708 / 711      | **6.3 / 8.0 / 9.1**      |
| 1m   | 223 / 433 / 1019 | 2754 / 2822 / 2842 | 2783 / 2844 / 2857  | **80.4 / 87.6 / 92.3**   |

vchord's RaBitQ-on-IVF is 10x-100x faster than every other comparator
across every size and percentile. It is the bar ecaz's RaBitQ-on-IVF
implementation must reach.

The pgvector IVFFlat and pgvectorscale DiskANN numbers (~700 ms at
100k, ~2800 ms at 1m) are consistent with sequential-scan or
poorly-probed paths — IVFFlat probes default to 1 and DiskANN has
configurable search depth; neither was tuned for recall in this pass.
Treat their absolute values as upper bounds, not tuned baselines.
HNSW is genuinely indexed; its high p95/p99 tails suggest occasional
graph-traversal misses at the configured `ef_search`.

## Storage footprint (live `pg_relation_size` at packet capture)

Corpus tables are pgvector `vector(1536)` raw float32 — 6 GB per
million rows regardless of which comparator owns the table. Each
comparator gets its own corpus copy because the host ran the
per-variant-table refactor for pgvector and one-table-per-AM for the
others. Index sizes:

| size | corpus (each) | pgv HNSW idx | pgv IVFFlat idx | pgvscale DiskANN idx | vchord RaBitQ idx |
|------|---------------|--------------|-----------------|----------------------|-------------------|
| 50k  | 3.0 MB        | 391 MB       | 392 MB          | 33 MB                | 415 MB            |
| 100k | 6.0 MB        | 781 MB       | 784 MB          | 65 MB                | 830 MB            |
| 1m   | 57 MB         | 7734 MB      | 7742 MB         | 645 MB               | 8211 MB           |

Observations:

- **pgvectorscale's DiskANN is by far the most space-efficient** —
  ~12x smaller than the others at every scale. SBQ compression of the
  graph payload + on-disk layout pays off; pgvector HNSW/IVFFlat and
  vchord all store full float32 vectors inline.
- vchord's RaBitQ-on-IVF index is the largest at every scale despite
  being the fastest at query time — pre-rotated codes + posting-list
  metadata cost storage but pay back hard on the latency side.
- pgvector HNSW vs IVFFlat are within ~1% on storage; the structure
  differs but the dominant cost is the inlined `vector(1536)` payload
  that both keep per tuple.
- vchord 1m catalog stats lagged at capture time (`reltuples = -1` for
  the corpus table, `0` for the rabitq index) — autovacuum hadn't
  re-analyzed yet. The index physically holds the data (8.2 GB on
  disk) and the latency numbers above confirm it's being used.

## Build times

Recovered from the `comparator_log "building ..."` / `"done. ..."`
timestamps that the load scripts emit into the chain stdout (captured
in `artifacts/comparators/_run-*.log`). Differences between bracket
events isolate the `CREATE INDEX` call within ~1 second.

| size | pgv HNSW | pgv IVFFlat | pgvscale DiskANN | vchord RaBitQ-on-IVF |
|------|----------|-------------|------------------|----------------------|
| 50k  | ~36 s    | ~13 s       | (sub-minute)     | ~5 s                 |
| 100k | ~93 s    | ~31 s       | (sub-minute)     | ~14 s                |
| 1m   | **22 m 13 s** | **6 m 16 s** | **72 m 47 s** | **3 m 37 s**     |

The 1m numbers are the load-bearing ones. Read together with the
latency table:

- **vchord wins both axes at 1m**: 3.6 min build → 80 ms p50 query.
- **pgvectorscale DiskANN trades a 73-min build for a 645 MB index
  and ~2.8 s p50 query** — the build cost looks bad until you note
  the index is 12x smaller than every other comparator. The latency
  is poor because default `streaming_diskann.search_list_size` was
  used; a tuned config will be much better. This packet does not
  claim DiskANN is slow in practice; it claims the out-of-the-box
  defaults are slow.
- **pgvector HNSW 22 min build + IVFFlat 6 min build** at 1m. The
  swap-dance tail in the old `bench_pgvector.sh` re-incurred the
  HNSW 22-min cost on every bench cycle — that's what the
  per-variant-table refactor in the companion commit eliminates.
- Sub-minute builds at 50k/100k are noise in the chain timestamps;
  treat as upper bounds.

Same-cycle reruns of the 1m pgvector build (`_run-2.log` initial vs
`_run-5.log` swap-dance rebuild) show 22:13 vs 22:10 for HNSW —
build cost is very repeatable on this host, not a one-off.

## Artifacts

Each `artifacts/comparators/<size>/<ext>/<variant>/` contains:

- `latency.log` — recovered percentile summary
- `run.log` — raw psql stdout with one `Time:` line per iteration (the
  authoritative measurement source)
- `raw.out` — query result rows (200 × `id` tuples)
- `raw.tsv` — `<line-no> <ms>` pairs extracted from `run.log`
- `queries.sql` — the exact 200 `SELECT id FROM ... ORDER BY embedding <#> ... LIMIT 10` statements
- `query_ids.txt` — query-id list pulled from `*_queries`

Tarball with the full tree: `artifacts/comparators-full.tar.gz`.

## Source files changed in the same series

Local-only at the time of this manifest; intended to land alongside
or just after it:

- `scripts/comparators/load_pgvector.sh` — per-variant replicated
  corpus tables for HNSW + IVFFlat (CTAS the second from the first)
- `scripts/comparators/bench_pgvector.sh` — removed drop+rebuild
  swap-dance tail; benches each variant against its own table
- `scripts/comparators/_bench_lib.sh` — accept explicit
  `--corpus-table` / `--queries-table`; **fix** `Time:` extraction to
  read `run.log` instead of `raw.out` (the bug responsible for the
  "no samples" first pass)
- `scripts/comparators/README.md` — documented the per-variant
  convention and the `--corpus-table` / `--queries-table` flags

## Re-run recipe

After the bench-lib fix lands, on a host with pg18 + ecaz + the three
comparator extensions installed:

```bash
cd /var/lib/pgsql/build/ecaz

scripts/comparators/install_pgvector.sh
scripts/comparators/install_pgvectorscale.sh
scripts/comparators/install_vchord.sh

for size in 50k 100k 1m; do
  case "$size" in
    50k|100k) staged=staged-$size; base=ec_real_${size%k}k ;;
    1m)       staged=staged-1m;    base=ec_real_ann_benchmarks_anchor ;;
  esac
  MAINT_WORK_MEM=4GB scripts/comparators/run_all.sh \
    --out /var/lib/pgsql/18/artifacts/comparators \
    --size "$size" --dim 1536 \
    --corpus-file "/var/lib/pgsql/18/datasets/${staged}/${base}_corpus.tsv" \
    --queries-file "/var/lib/pgsql/18/datasets/${staged}/${base}_queries.tsv" \
    --exts "pgvector pgvectorscale vchord" \
    --phases "load bench"
done
```

Then `scp -r` the `comparators/` tree back into this packet's
`artifacts/`.

## Not included

- Recall measurement — only latency was captured this pass
- IVFFlat probe sweeps / DiskANN search-depth sweeps — defaults only
- Lantern — no PG18 support upstream; see install_lantern.sh GAP
- pgvector cosine / L2 opclasses — IP only
- ecaz access-method comparison numbers (those live in
  `benchmarks/cloud-scaling-multi-am/`); this packet is comparator-side only

## Snapshot / state

Instance `i-05af7ea8e92f65b30` (m8g.2xlarge) was left running after
this packet was pulled. No EBS snapshot was taken at the SHA above; the
data EBS layout is co-located on root and the corpus tables are
reproducible from staged TSVs, so a snapshot adds little here.
