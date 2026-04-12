# Review Request: C1 Latency Launcher Plan Verification

## Context

Branch:
- `main`

Prior packets:
- `review/244-c1-real-corpus-latency-hardening/request.md`
- `review/245-c1-real-corpus-latency-10k-run/request.md`

Packet `244` hardened the real-corpus latency reporting surface so it recorded
`ef_search`, host / GUC metadata, and server-side throughput correctly.
Packet `245` then attempted the first real `10k` operator run on `main`.

That run exposed a second C1 problem:

- the launcher would happily spend minutes timing a representative query set
  without first checking whether the planner was actually using the requested
  tqhnsw index
- on current `main`, the representative plan on the loaded `tqhnsw_real_10k`
  fixture is:

```text
Limit
  ->  Sort
        Sort Key: (embedding <#> ...)
        ->  Seq Scan on tqhnsw_real_10k_corpus
```

So the attempted `NFR-001` run was measuring the wrong surface. This checkpoint
fixes that by adding a planner-verified launcher and by correcting the adjacent
docs / status surfaces so they no longer imply that durable HNSW latency
capture is already available on `main`.

## Scope

- `scripts/bench_sql_latency_verified.sh`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `docs/RECALL_REAL_CORPUS.md`
- `spec/non-functional/NFR-001-query-latency.md`
- `plan/tasks/10-benchmarks.md`
- `plan/status.md`

## What Landed

### 1. New planner-verified real-corpus launcher

`scripts/bench_sql_latency_verified.sh` is a guarded wrapper around
`scripts/bench_sql_latency.sh`.

Before it starts a long run, it:

1. requires a canonical `--prefix`
2. requires at most one effective `--m` per invocation
3. reads the first query vector from `<prefix>_queries`
4. runs a representative `EXPLAIN`
5. aborts unless the plan text includes the exact expected
   `<prefix>_m{N}_idx`

If the planner falls back to `Sort -> Seq Scan`, or if it chooses a different
tqhnsw index than the one requested for that run, the launcher prints the
representative plan and exits non-zero before timing anything.

The "one m per run" boundary is intentional: it keeps the expected index
unambiguous and prevents a multi-index table from silently benchmarking a
different `m` than the operator thought they were measuring.

### 2. Scratch wrapper for the verified launcher

`scripts/bench_sql_latency_verified_scratch.sh` mirrors the existing scratch
wrapper pattern but delegates to the verified launcher.

### 3. Docs / status now match the real block on `main`

The adjacent surfaces now say the honest thing:

- the planner-verified launcher exists
- current `main` still falls back to `Sort -> Seq Scan`
- durable HNSW latency artifacts remain blocked on planner-visible tqhnsw
  scans (or a dedicated forced-index benchmark seam)

That is a materially different state from "the loader/index surfaces are landed,
so trustworthy HNSW latency capture can start immediately."

## Validation

Script sanity:

- `bash -n scripts/bench_sql_latency_verified.sh`
- `bash -n scripts/bench_sql_latency_verified_scratch.sh`
- `bash scripts/bench_sql_latency_verified.sh --help`

Failure-path smoke on the loaded real `10k` fixture:

```bash
PGHOST=/home/peter/.pgrx \
PGPORT=28817 \
PGDATABASE=postgres \
TQV_PSQL_BIN=/home/peter/.pgrx/17.9/pgrx-install/bin/psql \
bash scripts/bench_sql_latency_verified.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --query-limit 1 \
    --cache-state smoke
```

Observed result:

- exit status: non-zero
- stderr begins with:

```text
planner verification failed for tqhnsw_real_10k_m8_idx
expected the representative plan to use tqhnsw_real_10k_m8_idx, but it did not.
aborting before timing so this launcher does not record Seq Scan + Sort
or the wrong tqhnsw index for the requested m value.
```

Required checkpoint validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All green on this checkpoint.

## Review Focus

- Is "one effective `m` per verified run" the right narrow boundary for
  planner-verified latency capture, given that a multi-index table cannot
  otherwise guarantee which index the planner will pick?
- Is a representative `EXPLAIN` preflight the right fail-fast boundary, or
  should the launcher also perform a second guard deeper into the run?
- Do the task / status / spec surfaces now describe the C1 state honestly:
  hardened and planner-verified launchers are landed, but durable HNSW latency
  artifacts on current `main` are still blocked on planner-visible tqhnsw
  scans?
