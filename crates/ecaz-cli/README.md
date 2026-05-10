# ecaz-cli

Operator CLI for the Ecaz Postgres vector extension. Single binary, single
entry point, profile-aware across access methods.

## Status

The CLI is now the supported operator surface for:

- corpus preparation, generation, loading, inspection, and listing
- recall / latency / storage / overhead benchmarks
- configured benchmark suites for long unattended runs
- DiskANN graph and build-probe diagnostics
- pgvector and pgvectorscale comparison
- quantizer feasibility studies
- HNSW and IVF stress validation
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
  `tqvector` / `ecvector` / `ec_hnsw` / `ec_ivf` / `ec_diskann`. Adding a new access
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

That installs the binary under Cargo's bin directory, usually
`$HOME/.cargo/bin/ecaz`. Interactive shells may also put that directory on
`$PATH`; when running from an agent or other sandboxed tool session, prefer the
absolute installed path (for example `/Users/peter/.cargo/bin/ecaz`) so one
approval rule can cover the operator surface consistently.

All commands accept `--database`, `--host`, `--port`, `--user`, `--password`,
and `--log-file`; each also falls back to the matching libpq environment
variable (`PGDATABASE`, `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`) when the
flag is omitted. `--host` may be either a TCP host name or a Unix socket
directory such as `/home/peter/.pgrx`.
`--log-file` mirrors the CLI's stdout/stderr into a packet-local artifact
file so review runs do not need shell `tee` wrappers. When `--log-file` is
set, transient progress bars are suppressed so the artifact stays stable.

## Sandboxed Agent Sessions

Use `ecaz` as the approval boundary for local PostgreSQL and pgrx work. In
practice that means granting the installed binary path once, then running corpus
generation, load/list/inspect, benchmark, storage, scratch, and SQL checks
through that binary:

```sh
/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres \
  --socket-dir /Users/peter/.pgrx --raw \
  --sql "select version()" \
  --log-output review/example/artifacts/pg18-status.log
```

Prefer this to direct `psql`, wrapper scripts, or one-off shell plumbing when
the operation already exists in the CLI. If a benchmark or setup step needs a
new repeated operation, add a narrow `ecaz` subcommand or option instead of
working around the sandbox with ad hoc commands. This keeps packet-local logs,
libpq options, and approval scope in one place.

External comparison extensions can use the same setup surface:

```sh
/Users/peter/.cargo/bin/ecaz dev install pgvector --pg 18
/Users/peter/.cargo/bin/ecaz dev install vectorscale \
  --pg 18 \
  --repo "$HOME/dev_bak/pgvectorscale/pgvectorscale" \
  --cargo-pgrx /tmp/pgvectorscale-cargo-pgrx-0.16.1/bin/cargo-pgrx
```

## Command tree

```
ecaz
├── corpus
│   ├── fetch       # fetch a canonical parquet release into a local directory
│   ├── load        # load a <basename>_{corpus,queries}.tsv fixture + build an index
│   ├── inspect     # show row counts, dim, indexes for a loaded corpus
│   ├── list        # enumerate corpora in the database
│   ├── generate    # synthesize unit-sphere TSV fixtures
│   └── prepare     # parquet -> canonical TSV + manifest
├── bench
│   ├── recall      # recall@k sweep against ground truth
│   ├── latency     # p50/p95/p99 SQL latency under concurrency
│   ├── storage     # table + index size accounting
│   ├── diskann-graph       # persisted graph reachability/degree/edge diagnostics
│   ├── diskann-build-probe # in-memory candidate pool/pruning/degree diagnostics
│   ├── overhead            # encode / internal scan / residual SQL breakdown
│   └── suite
│       ├── run             # dry-run or execute a configured benchmark suite
│       ├── audit           # validate suite shape and required local inputs
│       ├── status          # summarize a suite manifest
│       └── report          # emit a minimal markdown manifest report
├── compare
│   ├── pgvector    # side-by-side recall + latency vs pgvector
│   └── vectorscale # side-by-side DiskANN comparison vs pgvectorscale
├── dev
│   ├── install
│   │   ├── ecaz-pg-test # install the ecaz pg_test build into a pgrx tree
│   │   ├── pgvector     # install pgvector into the selected pg_config tree
│   │   └── vectorscale  # install pgvectorscale into the selected pg_config tree
│   ├── scratch
│   │   ├── restart               # restart pgrx with runtime env knobs
│   │   ├── sql                   # run psql against a pgrx scratch cluster
│   │   └── refresh-debug-helpers # install ADR-030 debug SQL wrappers
│   ├── sql         # version-aware pgrx SQL runner with packet-local logging
│   └── test
│       ├── pgrx                # run cargo pgrx test
│       └── pg18-preload-pgstat # validate PG18 preload/custom pgstat visibility
├── quant
│   └── feasibility # offline quantizer recall and error-bound calibration
└── stress
    ├── ivf-insert       # IVF live-insert throughput under workers
    ├── ivf-vacuum-scale # IVF VACUUM wall time, size, and RSS harness
    └── vacuum           # HNSW concurrent insert/delete/VACUUM invariant harness
```

Development SQL can be run through the CLI without shell redirection:

```sh
ecaz dev sql --pg 18 --file review/example/artifacts/run.sql --raw \
  --log-output review/example/artifacts/run.log
```

Use repeated `--env NAME=VALUE` flags to pass temporary environment to
the underlying `psql` process.

Corpus, benchmark, compare, and stress commands accept `--profile` where the
selected workflow is access-method specific. Current profiles are `ec_hnsw`,
`ec_ivf`, and `ec_diskann`, so a single corpus can be measured against multiple
access methods without re-loading data. Today all three profiles use `ecvector`
as the embedding column type, so one `<prefix>_corpus` table supports multiple
indexes side-by-side.

For the real-corpus path, the intended flow is now:

```sh
ecaz corpus fetch --output-dir /path/to/qdrant-dbpedia-openai3-1m
ecaz corpus prepare \
  --profile ec_hnsw_real_10k \
  --parquet /path/to/qdrant-dbpedia-openai3-1m/data \
  --output-dir /path/to/staged
ecaz corpus load \
  --log-file review/11073-task17-diskann-real-10k-recall/artifacts/load.log \
  --prefix ec_hnsw_real_10k \
  --corpus-file /path/to/staged/ec_hnsw_real_10k_corpus.tsv \
  --queries-file /path/to/staged/ec_hnsw_real_10k_queries.tsv \
  --profile ec_diskann
```

`ecaz corpus fetch` currently pins one first-class remote dataset:
Qdrant's DBpedia OpenAI `text-embedding-3-large` `1536`-dimensional
1M-row release on Hugging Face. It writes deterministic parquet shard
names under `<output-dir>/data/` plus a small `ecaz_fetch_manifest.json`
recording the source repo and revision.

## Access-method profiles

Profiles live in `src/profiles.rs` and describe:

- `access_method` — the `USING <am>` clause value.
- `operator_class` — the opclass used in `CREATE INDEX`.
- `embedding_type` — the column type used for the indexed expression.
- `encoder_function` — the SQL function that encodes `real[]` into that type.
- `encode_scan_query` — whether benchmark KNN probes should encode query
  parameters before `ORDER BY`; this is false for current opclasses because
  they declare `<#>(embedding, real[])`.
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
surface (`src/am/ec_hnsw/options.rs`, `src/am/ec_ivf/options.rs`,
`src/am/ec_diskann/options.rs`).
That's fine for three access methods and a small handful of knobs, but
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

`ecaz corpus list` shows what's loaded; `ecaz corpus inspect --prefix <prefix>`
shows the indexes built on it.

## Benchmark suites

`ecaz bench suite` runs longer benchmark plans from JSON config files. The
suite runner is intended for AM onboarding, tuning sweeps, repeatability
checks, and future RDS/Graviton runs where manual command sequences are too
easy to lose or mis-record.

Committed reusable suites live under [crates/ecaz-cli/suites](/Users/peter/dev/tqvector/crates/ecaz-cli/suites). The intended tiering is:

- `profile-cross-engine-real10k.json`: repeatable DiskANN/HNSW + pgvector/pgvectorscale comparison lane.
- `profile-hnsw-100k.json`: standard-machine HNSW lane with matched pgvector comparison.
- `profile-ivf-25k.json`: smaller-box IVF lane.
- `profile-ivf-50k.json`: mid-scale IVF lane.
- `profile-ivf-100k.json`: standard 64 GiB box IVF lane.
- `profile-ivf-1m.json`: opt-in scale IVF lane with suite-driven fetch + prepare + chunked load.

For a single repeatable entrypoint, use [scripts/run_benchmark_profile.sh](/Users/peter/dev/tqvector/scripts/run_benchmark_profile.sh):

```sh
scripts/run_benchmark_profile.sh standard --dry-run \
  --database postgres --host /Users/peter/.pgrx --port 28818
```

Tier guidance:

- `small`: weaker laptops that should avoid the 100k IVF lane.
- `standard`: 64 GiB development machines; includes real10k cross-engine, HNSW 100k, and IVF 100k.
- `full`: standard plus additional 25k and 50k IVF checkpoints.
- `scale`: full plus the suite-driven `1M` IVF lane. This is where the M5/64 GiB box should stretch, while weaker laptops should stay below it.

The first-supported config schema is JSON `schema_version: 1`:

```json
{
  "name": "task31-m5-ivf-100k",
  "schema_version": 1,
  "artifact_dir": "review/30178-task31-suite-runner-dry-run/artifacts",
  "defaults": {
    "profile": "ec_ivf",
    "bits": 4,
    "seed": 42,
    "queries_limit": 100,
    "iterations": 100,
    "force_index": true,
    "pg": 18,
    "socket_dir": "/Users/peter/.pgrx"
  },
  "thresholds": [
    {
      "name": "recall10-floor",
      "step": "recall10-nprobe-sweep-w500",
      "metric": "recall",
      "filters": {
        "nprobe": "96"
      },
      "field": "recall@k",
      "op": "gte",
      "value": 0.995
    }
  ],
  "steps": [
    {
      "kind": "recall",
      "name": "recall10-nprobe-sweep-w500",
      "tags": ["recall", "recall10", "sweep", "100k", "n128", "w500"],
      "prefix": "task31_m5_real100k_pqg8_n128",
      "k": 10,
      "sweep": [40, 48, 56, 64, 80, 96],
      "rerank_width": 500,
      "truth_cache_file": "review/example/artifacts/truth_k10.json",
      "log_output": "review/example/artifacts/recall10.log"
    }
  ]
}
```

Supported step kinds are:

- `corpus-fetch`: expands to `ecaz corpus fetch` so larger reusable suites can fetch their own parquet input tree.
- `corpus-prepare`: expands to `ecaz corpus prepare`, including optional `chunk_rows` for chunked large-corpus staging.
- `load`: expands to `ecaz corpus load`, including profile, corpus/query TSVs,
  optional manifest, chunked-manifest loading, native HNSW `m` /
  `ef_construction`, reloptions, and `--log-file`.
- `recall`: expands to `ecaz bench recall`, including `k`, sweep values,
  truth cache, query limit, rerank width, and `--log-output`.
- `latency`: expands to `ecaz bench latency`, including sweep values,
  iterations, concurrency, rerank width, memory sampling, and `--log-output`.
- `storage`: expands to `ecaz bench storage` with an optional `--log-file`.
- `explain`: generates the configured SQL file and runs it through
  `ecaz dev sql --raw --file ... --log-output ...`.
- `compare-pgvector`: expands to `ecaz compare pgvector`, including matched
  sweeps, pgvector HNSW build knobs, optional rebuild, and `--log-file`.
- `compare-vectorscale`: expands to `ecaz compare vectorscale`, including
  matched sweeps, pgvectorscale DiskANN build knobs, optional rebuild, and
  `--log-file`.
- `raw`: runs an explicit `args` array for a command not yet modeled by a
  first-class step kind. Use `expected_artifacts` if status/report should audit
  output files.

Before a long run, audit and dry-run the suite:

```sh
ecaz bench suite audit --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json

ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  bench suite run \
  --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json \
  --dry-run
```

To execute the full suite:

```sh
ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  bench suite run \
  --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json
```

The runner writes `suite-manifest.json` under `artifact_dir` unless
`--manifest-output` is provided. The manifest records the config path and
SHA256, redacted connection target, selected steps, expanded commands, expected
artifacts, status, timestamps, duration, and exit code. By default execution
stops on the first failed selected step; add `--continue-on-error` when a sweep
should keep going after failures.

During optimization, use `--only` to target a narrow slice while preserving the
same config, or `--only-tag` to target categories of steps:

```sh
ecaz bench suite run \
  --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json \
  --only recall10-nprobe-sweep-w500 \
  --only latency-nprobe-sweep-w500

ecaz bench suite run \
  --config crates/ecaz-cli/suites/task31-m5-ivf-100k.json \
  --only-tag recall \
  --only-tag candidate
```

Use `--resume-from <suite-manifest.json>` to skip steps that already succeeded
in a previous manifest while retaining the current config's expanded commands.
The runner writes normalized metric rows to `<artifact_dir>/results.jsonl` after
execution; override with `--results-output <path>` if the packet needs a
different location.

Optional `thresholds` fail a completed suite when parsed results miss a target.
Thresholds match by `step`, `metric`, optional exact-match `filters`, and result
`field`; supported operators are `gt`, `gte`, `lt`, `lte`, and `eq`. Numeric
parsing uses the leading number from fields such as `0.9980`, `10.8 ms`, or
`202.9 B`. Resume is strict: `--resume-from` only reuses succeeded step records
when the prior manifest config hash and expanded command match the current run.

After or during a run, inspect the manifest:

```sh
ecaz bench suite status \
  --manifest review/30178-task31-suite-runner-dry-run/artifacts/suite-manifest.json

ecaz bench suite report \
  --manifest review/30178-task31-suite-runner-dry-run/artifacts/suite-manifest.json \
  --results-output review/example/artifacts/results.jsonl
```

`status` reports completed, failed, skipped, dry-run, stale, and
missing-artifact counts. `report` emits a markdown summary from the manifest
and parses completed recall, latency, storage, and load artifacts into a
normalized JSONL result stream for plotting/comparison. Raw logs remain the
source of truth.

For RDS/Graviton or other remote runs, keep the same suite config shape and set
connection flags or libpq environment variables at invocation time. Store
hardware, instance class, storage, PostgreSQL settings, cache state, corpus,
query set, and command provenance in the review packet before making product
benchmark claims.

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
