# Comparator extension install scripts

These scripts install third-party PostgreSQL vector-search extensions so they
can be measured alongside ecaz on the same host. **Measurement itself is no
longer done here** — use the standalone, engine-generic CLI command instead:

```sh
ecaz bench comparator --engine {vchord|pgvector-hnsw|pgvector-ivfflat|pgvectorscale} \
  --prefix <corpus-prefix> --sweep <query-guc-values> [--lists N] ...
```

`ecaz bench comparator` builds the engine's sidecar table + index from the
ecaz `<prefix>_corpus.source` column, computes brute-force ground truth, and
emits recall@k + latency percentiles + storage per sweep value (one Pareto row
per value). It measures **one** external engine standalone — there is no ecaz
side, which keeps comparator measurement decoupled from ecaz re-measurement
(no-re-run policy). Drive matrices/sweeps through an `ecaz bench suite` config
with a `comparator` step (FR-038), not bash. See `crates/ecaz-cli/README.md`.

## Why these scripts remain

Installing an extension is an operator prerequisite the CLI cannot perform:
it requires copying files into the `pg_config` tree, editing
`shared_preload_libraries`, and restarting PostgreSQL. Each comparator keeps a
single idempotent `install.sh`:

```
pgvector/install.sh        # install pgvector into the selected pg_config tree
pgvectorscale/install.sh   # install pgvectorscale (StreamingDiskANN)
vchord/install.sh          # install VectorChord (vchordrq); preload + restart
```

`_common.sh` holds the shared helpers those install scripts source
(`comparator_log`, `comparator_extension_installed`, and the default
build/pg_config paths).

### vchord is the preload-and-restart case

`vchord/install.sh` downloads the prebuilt upstream zip for the host PG major +
arch, drops the `.so` / `.control` / `.sql` into the `pg_config` dirs, then
appends `vchord` to `shared_preload_libraries` via `ALTER SYSTEM` and restarts
PostgreSQL. Without the preload, `CREATE EXTENSION vchord` errors with
`vchord must be loaded via shared_preload_libraries`. After install:

```sh
scripts/comparators/vchord/install.sh
psql -c 'CREATE EXTENSION IF NOT EXISTS vchord CASCADE;'
ecaz bench comparator --engine vchord --prefix real_100k \
  --sweep 1,4,16,64 --lists 320 --log-output <packet>/artifacts/vchord-100k.log
```

## Operator cheatsheet

pgvector defines three distance operators; the comparator command and the
sidecar index DDL both use inner product (`<#>`, `vector_ip_ops`) to match
ecaz's IP semantics:

| Operator | Meaning | Used by |
|---|---|---|
| `<->` | L2 distance | pgvector L2 ops |
| `<#>` | negative inner product (`ORDER BY ... ASC`) | pgvector IP ops, vchord IP, ecaz |
| `<=>` | cosine distance | pgvector cosine ops |

Pick the operator that matches the opclass used when building the index;
mismatch falls back to a sequential scan.
