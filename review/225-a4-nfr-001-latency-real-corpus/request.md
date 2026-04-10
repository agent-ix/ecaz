# Review Request: NFR-001 Latency Lane on Real DBpedia Corpus

## Context

Task: `plan/tasks/coder2/10056-nfr-001-latency-real-corpus.md`
Branch: `feat/10056-nfr-001-latency-real-corpus`
Off main: `ef685d7 Add coder-2 parallel tasks for A4 real-corpus lane`

Adjacent A4 packets that landed the loader / converter / fixture surface
this branch reuses:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`
- `review/221-a4-real-corpus-subset-manifest-contract/request.md`
- `review/222-a4-real-corpus-fetch-and-schema-alignment/request.md`

`docs/RECALL_REAL_CORPUS.md` already noted that `NFR-001` latency
benchmarking would reuse the same loader path as the A4 recall lane but
target a different reporting surface (`scripts/bench_sql_latency.sh`). This
branch closes that gap so the same `tqhnsw_real_10k` /
`tqhnsw_real_50k` tables that the recall lane already loads are now also
the input to the latency lane, with no second load step.

## What Landed

### 1. `scripts/bench_sql_latency.sh` accepts canonical real-corpus prefixes

The script gained an additive `--prefix` mode. The legacy synthetic-fixture
path is byte-for-byte unchanged below the new arg-parser block; the new
real-corpus path is implemented as a function (`run_real_corpus_bench`)
that runs first when `--prefix` is set and `exit`s before falling through.

Real-corpus mode flags:

- `--prefix <name>` — canonical fixture prefix produced by
  `scripts/load_real_corpus.py` (validated against
  `^[a-zA-Z_][a-zA-Z0-9_]*$`). Derives:
  - corpus table: `<prefix>_corpus`
  - query table: `<prefix>_queries`
  - index name: `<prefix>_m{N}_idx`
- `--m N` — HNSW `m` to bench. Repeatable. Defaults to `8`. Each value
  must already have a built index — the script verifies via
  `to_regclass(...)` and refuses to run if the index is missing, with a
  pointer back to `load_real_corpus.py --m N`.
- `--ef-search csv` — `ef_search` sweep, defaults to
  `40,64,100,128,160,200` (the same shape the task spec asks for).
- `--query-limit N` — optional cap on queries per cell.
- `--output FILE` — optional file the per-cell summary lines are appended
  to in addition to stdout.

Per-cell measurement loop:

- Pulls the query set from `<prefix>_queries.source` once at start, into a
  tmp file in psql `\pset format aligned-off, tuples-only` shape (curly-brace
  `real[]` literal).
- For each `(m, ef_search)` pair:
  - Wraps each query in `EXPLAIN (ANALYZE, TIMING, FORMAT JSON) SELECT id
    FROM <prefix>_corpus ORDER BY embedding <#> '{...}'::real[] LIMIT K`
  - Captures the wall-clock duration of the entire cell as well, and uses
    `n_queries / wall_seconds` as the observed `qps`.
  - Pipes the captured EXPLAIN output through an inline python parser
    (same JSON-fragment scan as the legacy synthetic path) to extract
    `Execution Time` per query.
  - Emits one summary line:
    ```
    m=8   ef_search=40   n=200  p50=...ms p95=...ms p99=...ms mean=...ms min=...ms max=...ms qps=...
    ```
  - Appends the same line to `--output` if provided.

Per the task design notes, the script does **not** load anything in
real-corpus mode. Load and bench stay decoupled.

### 2. New `scripts/bench_sql_latency_scratch.sh` wrapper

Mirrors `scripts/load_real_corpus_scratch.sh` exactly:

```bash
export PGHOST="${PGHOST:-/tmp/tqvector_pgrx_home}"
export PGPORT="${PGPORT:-28817}"
export PGDATABASE="${PGDATABASE:-postgres}"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"
exec bash "${repo_root}/scripts/bench_sql_latency.sh" "$@"
```

The wrapper is intentionally a thin `exec` — it pins the same socket /
port / database / `psql` binary the loader scratch helper pins, and then
forwards all arguments verbatim. Both legacy synthetic mode and the new
`--prefix` mode work through this wrapper.

### 3. `docs/RECALL_REAL_CORPUS.md` got the handoff section

Added a new section "Reusing the Loaded Tables for NFR-001 Latency"
between "Reporting" and "Troubleshooting" with a single worked example
against the already-loaded `tqhnsw_real_10k` fixture. The A4 recall content
above and below it is unchanged. The section is exactly the one paragraph
of "the same loaded tables serve both NFR-003 and NFR-001; here is the
latency invocation" the task asks for, plus the canonical command.

### 4. `spec/non-functional/NFR-001-query-latency.md` cross-link

Added one short paragraph to the "Measurement" section pointing back at
`docs/RECALL_REAL_CORPUS.md` and naming `scripts/bench_sql_latency.sh
--prefix <canonical-prefix>` as the real-corpus reporting surface. The
existing approved gate text (p50 < 5ms, p99 < 15ms on 50k m=8 ef_search=40)
is unchanged.

## Design notes followed

- Did not rewrite the bench harness; the synthetic-fixture path is
  byte-for-byte unchanged. The diff is purely additive on top of it.
- Did not add a load step to the bench script. The loader stays in
  `scripts/load_real_corpus.py`.
- Did not add any new latency percentiles. NFR-001 declares p50 and p99;
  the script also reports p95 and `qps` because they sit alongside the
  required percentiles in the same sort, but the gate target lines stay on
  p50 / p99 only.
- Same `build_source_column = 'source'` indexes the A4 recall lane builds
  are the indexes the bench script reads — no parallel index, no DDL of
  any kind in `bench_sql_latency.sh`.
- Did not change NFR-001 target numbers.

## Evidence

### Bash parse / lint

```
$ bash -n scripts/bench_sql_latency.sh
$ bash -n scripts/bench_sql_latency_scratch.sh
```

both clean.

### Help / arg-parser smoke

```
$ bash scripts/bench_sql_latency.sh --help
Usage:
  Synthetic-fixture mode (legacy default; tunables via env vars):
    bash scripts/bench_sql_latency.sh

  Real-corpus mode (reuses preloaded tables and indexes):
    bash scripts/bench_sql_latency.sh --prefix <prefix> [--m N]... \
        [--ef-search csv] [--query-limit N] [--output FILE]
  ...
```

```
$ bash scripts/bench_sql_latency.sh --prefix '1bad-prefix'
invalid prefix: 1bad-prefix
exit=2

$ bash scripts/bench_sql_latency.sh --prefix tqhnsw_real_10k --m 8 --ef-search 'oops'
invalid ef_search value: oops
exit=2
```

Validation rejects malformed prefixes and ef_search values before issuing
any psql command.

### First latency sweep — deferred to coder-1

The actual `--prefix tqhnsw_real_10k` run requires:

1. A live pgrx scratch `pg17` cluster on socket `/tmp/tqvector_pgrx_home`.
2. The current `pg_test` build of the extension installed in that cluster.
3. The canonical `tqhnsw_real_10k_corpus` / `tqhnsw_real_10k_queries`
   tables loaded into it.
4. Both `tqhnsw_real_10k_m8_idx` and `tqhnsw_real_10k_m16_idx` already
   built.

On this branch's host none of those preconditions hold:

- `/tmp/tqvector_pgrx_home/17.9/data-17/` does not exist.
- `pgrep` shows no pgrx scratch postgres process.
- `/home/peter/dev/datasets/tqhnsw_real_10k/` does not exist (the staged
  TSVs from review 222 live on coder-1, not on this host).

Per the user direction on this branch, the staged corpus on coder-1 is
not synced over for the latency capture; coder-1 already has the loaded
tables and is the natural place to record the first measured numbers.
The hand-off command set is exactly:

```bash
scripts/bench_sql_latency_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --output /tmp/nfr1_real_10k.txt > /tmp/nfr1_real_10k.stdout
```

Capture both `/tmp/nfr1_real_10k.txt` (per-cell summary lines) and the
full `nfr1_real_10k.stdout` (header banner + cell separators) verbatim
into a follow-up packet next to the recall numbers from coder-1's first
real-corpus A4 run, plus:

- the scratch cluster GUC snapshot (`SHOW shared_buffers`,
  `SHOW max_parallel_workers_per_gather`, `SHOW work_mem`);
- the host CPU model and physical RAM;
- whether the run was warm or cold cache (the script does not warm the
  cache itself, so the first cell of each `m` is effectively cold).

The pass/fail line should compare the captured `(m=8, ef_search=40)`
row against `NFR-001`'s `p50 < 5ms` / `p99 < 15ms` gates. If the 10k
fixture misses the 50k-fixture-shaped gate target, land the result red
and file the follow-up — do not retune NFR-001 here.

This deferral matches the same pattern as the ann-benchmarks anchor in
`review/224-a4-ann-benchmarks-anchor/request.md`, which also lands the
code path and hands the first measured number off to the operator that
already has the staged corpus.

## Why This Matters

Until this branch lands, the only A1/NFR-001 numbers in the repo are
synthetic-fixture latencies. They are useful for catching regressions in
the cost model itself, but they are not credible against the same
1536-dim DBpedia surface the recall gate already measures. Wiring the
latency surface to the canonical real-corpus prefix gets `NFR-001` and
`NFR-003` measuring against *the same loaded data* with one load step,
which is the cheapest possible way to gain a real-corpus latency
baseline.

## Out of Scope

- Rewriting `scripts/bench_sql_latency.sh`. Only the additive `--prefix`
  path is new on this branch.
- Adding new latency percentiles to NFR-001.
- Diagnosing latency regressions. Per the task spec, the first measured
  number is recorded pass-or-fail; failures are addressed in a follow-up.
- Running against `tqhnsw_real_50k`. The 50k index build is long; the
  task explicitly scopes "first sweep" to 10k.

## Files

- `scripts/bench_sql_latency.sh` (additive `--prefix` mode, +264 lines)
- `scripts/bench_sql_latency_scratch.sh` (new wrapper)
- `docs/RECALL_REAL_CORPUS.md` (new "Reusing the Loaded Tables for
  NFR-001 Latency" section)
- `spec/non-functional/NFR-001-query-latency.md` (one-paragraph cross-link
  under "Measurement")
