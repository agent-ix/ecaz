# Review Request: C1 Task16 Args-Only Scratch Wrapper Targeting

Current head at execution: `50f62ad`

## Context

Task 16 still needs live lever-4 / lever-5 runtime measurements on the pg17
scratch lane. The user explicitly required that those measurements run through
approved script prefixes without env-prefixed shell invocations, because
`FOO=bar ./script.sh ...` trips the approval gate.

The existing scratch helpers were not fully standardized for that workflow:

- some wrappers only supported cluster targeting through ambient env
- runtime experiment knobs on restart also depended on ambient env

This slice standardizes the missing generic surfaces instead of adding
task-specific one-offs.

## Code Changes

### 1. Added explicit cluster-targeting args to scratch wrappers

These wrappers now accept:

- `--socket-dir DIR`
- `--port PORT`

Updated scripts:

- `scripts/pg17_scratch_psql.sh`
- `scripts/bench_sql_latency_scratch.sh`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `scripts/load_real_corpus_scratch.sh`
- `scripts/run_real_corpus_recall_scratch.sh`

That allows scratch commands to target the intended cluster through args instead
of env-prefixed invocation.

### 2. Added generic runtime env forwarding to scratch restart

`scripts/restart_adr030_scratch.sh` now accepts repeated:

- `-e NAME=VALUE`
- `--env NAME=VALUE`

This is intentionally generic. It allows runtime experiment controls such as
`TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE=int8_approx` without adding bespoke flags
for each new measurement seam.

### 3. Added wrapper coverage

Updated `scripts/tests/test_pg17_scratch_psql_socket_resolution.py` so the
wrapper test also covers the new `--socket-dir` argument path.

## Why This Slice Exists

This is not task-16 scorer plumbing. It is the minimum generic script work
needed so the remaining measurements can run through standardized approved
script forms:

- explicit args for cluster/socket targeting
- generic forwarded env for restart-time runtime knobs

Without this slice, the remaining measurements were still falling back to the
approval-gated env-prefix pattern the user asked to stop using.

## Validation

Ran on this exact tree:

```bash
python3 scripts/tests/test_pg17_scratch_psql_socket_resolution.py
bash -n scripts/pg17_scratch_psql.sh scripts/bench_sql_latency_scratch.sh scripts/bench_sql_latency_verified_scratch.sh scripts/load_real_corpus_scratch.sh scripts/run_real_corpus_recall_scratch.sh scripts/restart_adr030_scratch.sh
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

All passed.

## Next Step

Use the standardized script surfaces above to run the remaining task-16
TurboQuant option measurements without further shell-level env-prefix work.
