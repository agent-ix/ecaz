# ecaz-cli

Operator CLI for the Ecaz Postgres vector extension. Single binary, single
entry point, profile-aware across access methods.

## Status

**v1 foundation PR (current):** CLI skeleton, full `--help` tree, foundation
modules (`profiles`, `psql`, `reloptions`) with tests. All commands declared,
but only the skeleton is implemented — every subcommand currently exits
with `not yet implemented`.

**v2 PRs (follow-up):** port the Python/shell scripts command-by-command.
Each port is its own PR so review stays tractable.

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
`PGDATABASE`) and inherit libpq environment variables (`PGHOST`, `PGPORT`,
`PGUSER`, `PGPASSWORD`) for connection setup.

## Command tree

```
ecaz
├── corpus
│   ├── load        # load a <basename>_{corpus,queries}.tsv fixture + build an index
│   ├── inspect     # show row counts, dim, indexes for a loaded corpus
│   └── list        # enumerate corpora in the database
├── bench
│   ├── recall      # recall@k sweep against ground truth
│   ├── latency     # p50/p95/p99 SQL latency under concurrency
│   ├── storage     # table + index size accounting
│   └── overhead    # encode / internal scan / residual SQL breakdown
├── compare
│   └── pgvector    # side-by-side recall + latency vs pgvector
└── stress
    └── vacuum      # concurrent insert/delete/VACUUM invariant harness
```

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
```

Unit tests cover `profiles`, `reloptions`, and the SQL builders in
`psql`. Integration tests against a live Postgres land alongside each
command port.
