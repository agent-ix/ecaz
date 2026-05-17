# Review Request: C1 Task16 Scratch Wrapper DB Targeting

Current head at execution: `13b7368`

## Context

The current task-16 benchmark seam needs a clean current-head scratch database.
The existing `postgres` scratch DB still contains the pre-refactor extension
surface, and the user explicitly asked that all measurement commands run
through script args rather than env-prefixed shell invocations.

The missing generic surface was database targeting on the scratch wrappers.

## What changed

Added `--db DB` support to the scratch wrappers that participate in load and
measurement flows:

- `scripts/load_real_corpus_scratch.sh`
- `scripts/bench_sql_latency_scratch.sh`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `scripts/run_real_corpus_recall_scratch.sh`
- `scripts/bench_pgvector_sql_latency_scratch.sh`

Behavior:

- wrappers now accept `--db DB`
- the chosen DB is exported as `PGDATABASE` for delegate scripts
- `run_real_corpus_recall_scratch.sh` forwards `--db` explicitly into
  `scripts/pg17_scratch_psql.sh`

This keeps scratch targeting on the args-only path instead of relying on
ambient env or env-prefixed script invocation.

## Why this slice exists

This is not ANN/runtime logic. It is the minimum generic script work needed to:

- create a clean current-head benchmark DB
- load the real corpus there
- run latency and recall scripts there
- keep the measurement path aligned with the user’s “no env vars in front of
  scripts” requirement

## Validation

Ran on this exact tree:

```bash
bash -n scripts/load_real_corpus_scratch.sh scripts/bench_sql_latency_scratch.sh scripts/bench_sql_latency_verified_scratch.sh scripts/run_real_corpus_recall_scratch.sh scripts/bench_pgvector_sql_latency_scratch.sh
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

All passed.

## Next step

Use `--db` to benchmark the canonical `ecvector` row model on a fresh
current-head scratch database, then packet the actual task-16 measurement
results separately.
