# Comparator extension scripts

Per-comparator scripts for benchmarking third-party PostgreSQL
vector-search extensions alongside ecaz on the same host.

Each comparator (`pgvector`, `pgvectorscale`, `vchord`, `lantern`,
...) is **fully independent**. To add a new one, write three files
following the same pattern:

```
install_<name>.sh   # builds the extension from source
load_<name>.sh      # creates real_<size>_<name>_corpus + builds its index
bench_<name>.sh     # runs latency bench, writes <out>/<size>/<name>/<idx>/
```

Shared helpers live in:
- `_common.sh` — log helper, extension-installed checks, vector-table loader, nlists heuristic
- `_bench_lib.sh` — the `comparator_bench_latency` helper used by every bench script

The orchestrator `run_all.sh` is a thin convenience wrapper that
calls each per-comparator script in sequence; it's **not** where
comparator-specific behavior lives.

## Adding a new comparator

1. Add `install_<name>.sh` that handles "build from source if not
   already installed". Use `comparator_extension_installed <control-name>`.
2. Add `load_<name>.sh` that creates `real_<size>_<name>_corpus`
   (use `comparator_load_vector_table`) and builds the extension's
   recommended index via `CREATE INDEX`. Be idempotent.
3. Add `bench_<name>.sh` that calls `comparator_bench_latency` with
   the right operator (`<#>` IP, `<->` L2, `<=>` cosine).
4. Optionally extend `run_all.sh`'s case statements if you want it in
   the convenience orchestrator.

## Operator cheatsheet

pgvector defines three distance operators:

| Operator | Meaning | Used by |
|---|---|---|
| `<->` | L2 distance | pgvector L2 ops |
| `<#>` | negative inner product (use `ORDER BY ... ` ASC) | pgvector IP ops, vchord IP, ecaz ec_ivf |
| `<=>` | cosine distance | pgvector cosine ops, pgvectorscale, lantern |

Pick the operator that matches the opclass you used when building
the index. Mismatch = sequential scan.

## Reproduction recipe

From a fresh bench host with pg18 + ecaz installed:

```bash
cd /var/lib/pgsql/build/ecaz

# Install everything
scripts/comparators/install_pgvector.sh
scripts/comparators/install_pgvectorscale.sh
scripts/comparators/install_vchord.sh
scripts/comparators/install_lantern.sh

# Load corpus + build indexes for one size
for sh in scripts/comparators/load_*.sh; do
  $sh --size 1m --dim 1536 \
      --corpus-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_corpus.tsv \
      --queries-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_queries.tsv
done

# Bench
for sh in scripts/comparators/bench_*.sh; do
  $sh --out /var/lib/pgsql/18/artifacts/comparators --size 1m
done

# Or use the orchestrator for the same effect:
scripts/comparators/run_all.sh \
    --out /var/lib/pgsql/18/artifacts/comparators \
    --size 1m --dim 1536 \
    --corpus-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_corpus.tsv \
    --queries-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_queries.tsv
```
