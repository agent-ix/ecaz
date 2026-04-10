# Review Request: A4 Real-Corpus Loader Smoke

## Context

Branch:
- `fix/a4-real-corpus-recall-lane`

Related packet:
- `review/218-a4-real-corpus-recall-lane/request.md`

Goal of this follow-up:
- Verify the new real-corpus loader on an actual scratch `pg17` cluster, not
  just through the Rust `#[pg_test]` wrapper.
- Record the setup failures encountered on the way there so the next rerun
  does not rediscover them.

## What I Changed

### 1. Explicit `psql` resolution in the loader

Updated `scripts/load_real_corpus.py` so it no longer assumes `psql` is on
`PATH`. It now:
- prefers `TQV_PSQL_BIN` when set
- otherwise falls back to `shutil.which("psql")`
- otherwise fails with a direct actionable error

Why this mattered:
- in the sandboxed exec environment, the first real loader run failed with:
  - `error: [Errno 2] No such file or directory: 'psql'`

### 2. Scratch-cluster helper

Added:
- `scripts/load_real_corpus_scratch.sh`

This pins the repo-local scratch defaults that already exist elsewhere in the
lane:
- `PGHOST=/tmp/tqvector_pgrx_home`
- `PGPORT=28817`
- `PGDATABASE=postgres`
- `TQV_PSQL_BIN=/home/peter/.pgrx/17.9/pgrx-install/bin/psql`

This is the equivalent convenience wrapper for the loader path that
`scripts/pg17_scratch_psql.sh` already provides for direct SQL probes.

### 3. Docs note for `pg_test` + explicit `psql`

Updated `docs/RECALL_REAL_CORPUS.md` to note:
- the external SQL surfaces require the `pg_test` install
- `TQV_PSQL_BIN` is the supported escape hatch when `psql` is not on `PATH`
- the scratch helper exists for repo-local reruns

## Failed / Partial Attempts

### A. Sandboxed loader, no `psql` on `PATH`

Command shape:

```bash
PGHOST=/tmp/tqvector_pgrx_home PGPORT=28817 PGDATABASE=postgres \
python3 scripts/load_real_corpus.py ...
```

Result:
- failed immediately with `No such file or directory: 'psql'`

Interpretation:
- loader logic had not run far enough to test database behavior yet
- this was an environment/setup gap, not a SQL/load bug

### B. Sandboxed direct `psql` against scratch socket

Command shape:

```bash
PGHOST=/tmp/tqvector_pgrx_home PGPORT=28817 \
/home/peter/.pgrx/17.9/pgrx-install/bin/psql postgres -c "select 1"
```

Result:
- failed with:
  - `connection to server on socket "/tmp/tqvector_pgrx_home/.s.PGSQL.28817" failed: Operation not permitted`

Interpretation:
- the scratch socket path is blocked in the sandbox for direct subprocess use
- top-level approved helpers work, but the Python loader must run outside the
  sandbox to exercise the real path

### C. Stale same-version `pg_test` install in the scratch DB

After `cargo pgrx install --release --test --pg-config ... --features 'pg17 pg_test'`,
the scratch `postgres` database still lacked:
- `tests.tqhnsw_graph_scan_recall_external_gate_report`
- `tests.tqhnsw_graph_scan_recall_external_summary`

Why:
- the extension was already installed in that DB from an earlier same-version
  build
- reinstall updated the SQL file on disk, but did not refresh objects already
  created in the database

Fix used for this smoke:

```sql
DROP EXTENSION IF EXISTS tqvector CASCADE;
CREATE EXTENSION tqvector CASCADE;
```

Note:
- on the scratch cluster this is fine
- on a reused DB this will cascade to dependent test relations, so either use a
  clean DB or plan for a reload

## End-to-End Smoke Run

### Fixture

Created a deterministic local TSV pair in `/tmp`:
- corpus: `24` rows
- queries: `6` rows
- dimensionality: `1536`
- format: `<id>\t<json_array>`

### Loader command

```bash
./scripts/load_real_corpus_scratch.sh \
  --prefix tqhnsw_loader_smoke_v2 \
  --corpus-file /tmp/tqhnsw_loader_smoke_corpus.tsv \
  --queries-file /tmp/tqhnsw_loader_smoke_queries.tsv \
  --m 8 16
```

Result:
- inserted corpus rows
- encoded `embedding`
- inserted query rows
- built `m=8` and `m=16` indexes
- completed successfully

Observed loader output:

```text
[loader] inserting corpus rows into tqhnsw_loader_smoke_v2_corpus ...
[loader] encoding tqvector embedding column for 24 rows in tqhnsw_loader_smoke_v2_corpus ...
[loader] inserting query rows into tqhnsw_loader_smoke_v2_queries ...
[loader] building tqhnsw_loader_smoke_v2_m8_idx (m=8, ef_construction=128) ...
[loader] building tqhnsw_loader_smoke_v2_m16_idx (m=16, ef_construction=128) ...
[loader] done. corpus=tqhnsw_loader_smoke_v2_corpus (24 rows), queries=tqhnsw_loader_smoke_v2_queries (6 rows), m=[8, 16]
```

### Idempotent rerun

Ran the same command again.

Result:
- skipped corpus reload
- skipped query reload
- skipped both index builds

Observed rerun output:

```text
[loader] tqhnsw_loader_smoke_v2_corpus already has 24 rows; skipping reload
[loader] tqhnsw_loader_smoke_v2_queries already has 6 rows; skipping reload
[loader] tqhnsw_loader_smoke_v2_m8_idx already exists with m=8 ef_construction=128; skipping rebuild
[loader] tqhnsw_loader_smoke_v2_m16_idx already exists with m=16 ef_construction=128; skipping rebuild
[loader] done. corpus=tqhnsw_loader_smoke_v2_corpus (24 rows), queries=tqhnsw_loader_smoke_v2_queries (6 rows), m=[8, 16]
```

## SQL Verification

### Loaded rows

```sql
SELECT
  (SELECT count(*) FROM tqhnsw_loader_smoke_v2_corpus),
  (SELECT count(*) FROM tqhnsw_loader_smoke_v2_queries);
```

Result:
- `24 | 6`

### Built indexes

```sql
SELECT relname, reloptions
FROM pg_class
WHERE relname IN ('tqhnsw_loader_smoke_v2_m8_idx', 'tqhnsw_loader_smoke_v2_m16_idx')
ORDER BY relname;
```

Result:
- `tqhnsw_loader_smoke_v2_m16_idx | {m=16,ef_construction=128,build_source_column=source}`
- `tqhnsw_loader_smoke_v2_m8_idx  | {m=8,ef_construction=128,build_source_column=source}`

### External gate report

```sql
SELECT *
FROM tests.tqhnsw_graph_scan_recall_external_gate_report(
  'tqhnsw_loader_smoke_v2_corpus',
  'tqhnsw_loader_smoke_v2_queries',
  'tqhnsw_loader_smoke_v2'
)
ORDER BY m, ef_search;
```

Result:

```text
8   |  40 | 0.93333334 |       | t
8   | 128 | 0.93333334 | 0.89  | t
8   | 200 | 0.93333334 |       | t
16  | 200 | 0.93333334 |       | t
```

### External summary

```sql
SELECT corpus_rows, query_count, graph_recall_at_10,
       exact_quantized_recall_at_10, ndcg_at_10, mean_abs_score_error
FROM tests.tqhnsw_graph_scan_recall_external_summary(
  'tqhnsw_loader_smoke_v2_corpus',
  'tqhnsw_loader_smoke_v2_queries',
  'tqhnsw_loader_smoke_v2_m8_idx',
  8,
  128
);
```

Result:
- `24 | 6 | 0.93333334 | 0.93333334 | 0.99119234 | 8.020205`

## Current Read

The real-corpus loader lane is now proven on the actual local path:
- loader can ingest staged TSV data
- loader can build the expected tqhnsw indexes
- reruns are idempotent
- the external SQL recall surfaces work against the loaded relations

The remaining setup caveat is operational, not architectural:
- on a reused scratch DB, reinstalling a same-version `pg_test` build does not
  refresh already-created SQL objects
- either use a fresh DB, or drop/recreate the extension before first use

## Files

- `scripts/load_real_corpus.py`
- `scripts/load_real_corpus_scratch.sh`
- `docs/RECALL_REAL_CORPUS.md`
