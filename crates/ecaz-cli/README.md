# ecaz-cli

Operator CLI for the Ecaz Postgres vector extension. Single binary, single
entry point, profile-aware across access methods.

## Status

The CLI is now the supported operator surface for:

- corpus preparation, generation, loading, inspection, and listing
- recall / latency / storage / overhead benches
- pgvector comparison
- vacuum stress validation
- scratch-cluster control and local development/test helpers

Wrapper scripts that only forwarded into one of those surfaces are being
removed from `main`. The remaining live shell entry points are either
`make` aliases or truly generic repo helpers that do not have a better
home in the operator CLI.

## Why another CLI

The `scripts/` directory grew organically as each benchmark was added: each
one duplicated argument parsing, psql plumbing, reloption quoting, and its
own notion of "what is an HNSW index." `ecaz-cli` replaces that with:

- One entry point (`ecaz`), one `--help` tree.
- A declarative `IndexProfile` registry — commands don't hardcode
  `tqvector` / `ecvector` / `ec_hnsw` / `ec_diskann`. Adding a new access
  method is one entry in `profiles.rs` plus whatever AM-specific logic
  the new method needs.
- Async Postgres (`tokio-postgres`) instead of shelling out to `psql`.
- Rich terminal output (`indicatif`, `comfy-table`) with consistent
  progress and results formatting.

## Install

From the repository root:

```sh
cargo install --path crates/ecaz-cli
```

That puts `ecaz` on `$PATH`. All commands accept `--database` (or read
`PGDATABASE`) and the CLI now exposes explicit connection flags
(`--host`, `--port`, `--user`, `--password`) in addition to the libpq
environment variables.

## Command tree

```
ecaz
├── corpus
│   ├── load        # load a <basename>_{corpus,queries}.tsv fixture + build an index
│   ├── inspect     # show row counts, dim, indexes for a loaded corpus
│   ├── list        # enumerate corpora in the database
│   ├── generate    # synthesize unit-sphere TSV fixtures
│   └── prepare     # parquet -> canonical TSV + manifest
├── bench
│   ├── recall      # recall@k sweep against ground truth
│   ├── latency     # p50/p95/p99 SQL latency under concurrency
│   ├── storage     # table + index size accounting
│   └── overhead    # encode / internal scan / residual SQL breakdown
├── compare
│   └── pgvector    # side-by-side recall + latency vs pgvector
├── dev
│   ├── install     # local ecaz/pgvector install helpers
│   ├── scratch     # pgrx scratch cluster restart/sql/refresh helpers
│   ├── sql         # version-aware pgrx SQL runner with packet-local logging
│   └── test        # pgrx and preload-aware validation lanes
└── stress
    └── vacuum      # concurrent insert/delete/VACUUM invariant harness
```

Development SQL can be run through the CLI without shell redirection:

```sh
ecaz dev sql --pg 18 --file review/example/artifacts/run.sql --raw \
  --log-output review/example/artifacts/run.log
```

Use repeated `--env NAME=VALUE` flags to pass temporary environment to
the underlying `psql` process.

Each command accepts `--profile` (e.g. `ec_hnsw`, `ec_diskann`) so a
single corpus can be measured against multiple access methods without
re-loading data. Today `ec_hnsw` and `ec_diskann` both use `ecvector` as
the embedding column type, so one `<prefix>_corpus` table supports both
indexes side-by-side.

## Access-method profiles

Profiles live in `src/profiles.rs` and describe:

- `access_method` — the `USING <am>` clause value.
- `operator_class` — the opclass used in `CREATE INDEX`.
- `embedding_type` — the column type used for the indexed expression.
- `encoder_function` — the SQL function that encodes `real[]` into that type.
- `ef_search_guc` — the per-scan tuning GUC name.
- `build_source_column` — whether the AM can index from a raw `real[]`
  column (HNSW can; DiskANN reads the indexed column directly).
- `sweep_axis` — which parameter the CLI should sweep by default
  (`m` for HNSW, `list_size` for DiskANN, `None` for AMs without a
  natural single axis).
- `known_reloptions` — reloption keys the CLI knows about. Unknown keys
  are still accepted via `--reloption key=value` passthrough; this set
  is for help text and light validation only.

### Drift risk, and the plan

The CLI currently hand-mirrors the extension's opclass and reloption
surface (`src/am/ec_hnsw/options.rs`, `src/am/ec_diskann/options.rs`).
That's fine for two access methods and a small handful of knobs, but
won't stay fine as the surface grows.

The planned fix — deferred to a follow-up PR — is to extract a
`crates/ecaz-core` crate exporting the shared constants and bounds, and
have both the extension (pgrx) and this CLI depend on it. Then the CLI
cannot drift from the extension: adding `use_pq_rerank: bool` to DiskANN
will either compile everywhere or break the build.

For v1 we accept the small duplication. If drift bites before then,
promote the extraction PR.

## Multiple corpora

A "corpus" is a named fixture identified by `--prefix`. Each corpus
occupies:

- `<prefix>_corpus`  — the row table (id, source real[], embedding).
- `<prefix>_queries` — the query-set table (id, source real[]).
- Any number of indexes on `<prefix>_corpus.embedding`.

Many corpora coexist in one database. Benchmarks target them by prefix:

```sh
ecaz corpus load --prefix dbpedia_10k   --corpus-file dbpedia_10k_corpus.tsv   ...
ecaz corpus load --prefix cohere_100k   --corpus-file cohere_100k_corpus.tsv   ...
ecaz corpus load --prefix dbpedia_10k --profile ec_diskann   ...  # adds a second index on the same corpus
```

`ecaz corpus list` shows what's loaded; `ecaz corpus inspect <prefix>`
shows the indexes built on it.

## Performance notes

- Brute-force ground truth for `bench recall` uses `ndarray` + `rayon`.
  At 1M rows × 1536 dim this lands around 1–3 seconds per 100-query
  batch — roughly 95% of what FAISS (C++) would give us, with zero
  non-Rust build deps.
- TSV ingest (`corpus load`) streams with `tokio::io::BufReader` +
  `COPY FROM STDIN`. Handles 10M-row corpora in seconds, not minutes.

If either ever becomes the bottleneck, the escape hatch is a FAISS-FFI
dep (via the `faiss` crate) for ground truth, or `simd-json` for ingest.
Neither is needed today.

## Testing

```sh
cargo test -p ecaz-cli
cargo pgrx test pg18
ecaz dev test pgrx
ecaz dev test pgrx --pg 18
ecaz dev test pg18-preload-pgstat
```

Unit tests cover `profiles`, `reloptions`, and the SQL builders in
`psql`, and the dev/test helpers now own the old wrapper-script validation
surface directly.
