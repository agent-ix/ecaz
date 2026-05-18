# Review Request: C1 ADR-030 V2 Scratch Socket Target Guardrails

## Context

Packet `401` promoted the proven `PqFastScan` runtime lane into the code
defaults:

- live rerank window defaults to `64`
- traversal score mode defaults to binary

During the measurement work that motivated packet `401`, I hit a more basic
operational problem:

- the "scratch" wrappers silently fell back from `/tmp/tqvector_pgrx_home` to
  `~/.pgrx` whenever the `/tmp` socket was missing
- that made it easy to think a command was targeting the scratch cluster while
  actually measuring a different postmaster

That is exactly what happened on 2026-04-16 during the real-corpus reruns.
The branch recovered the correct interpretation, but the wrapper behavior made
the mistake too easy.

## Problem

Before this packet, the scratch wrappers encoded an unsafe policy:

1. prefer `/tmp/tqvector_pgrx_home`
2. silently use `~/.pgrx` if that socket exists
3. otherwise still point at `/tmp/tqvector_pgrx_home`

That is fine for convenience and wrong for measurement hygiene. The caller
cannot tell from the wrapper invocation alone which cluster will actually be
used.

For the ADR030 / task-15 landing work, that ambiguity is unacceptable because:

- the real-corpus tables and indexes may live on one cluster
- the current extension install and runtime env may live on another
- scratch benchmarking commands need to fail loudly when that state is
  ambiguous, not guess

## Planned Slice

One operational checkpoint:

1. centralize scratch socket resolution in one helper
2. default all scratch wrappers to `/tmp/tqvector_pgrx_home`
3. refuse a silent `~/.pgrx` fallback
4. keep explicit targeting available through:
   - `TQV_PG_SOCKET_DIR`
   - `PGHOST`
5. add a focused regression test for that policy

No AM/runtime behavior change.

## Implementation

Updated:

- `scripts/resolve_scratch_socket_dir.sh`
- `scripts/pg17_scratch_psql.sh`
- `scripts/load_real_corpus_scratch.sh`
- `scripts/bench_sql_latency_scratch.sh`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `scripts/bench_pgvector_sql_latency_scratch.sh`
- `scripts/bench_tqvector_sql_overhead_breakdown_scratch.sh`
- `scripts/tests/test_resolve_scratch_socket_dir.py`

Concrete changes:

1. added `scripts/resolve_scratch_socket_dir.sh`
   - returns `TQV_PG_SOCKET_DIR` when explicitly set
   - otherwise returns `PGHOST` when explicitly set
   - otherwise requires the preferred scratch socket at
     `/tmp/tqvector_pgrx_home/.s.PGSQL.<port>`
   - if only `~/.pgrx` exists, errors with an explicit "refusing to fall back"
     message
2. replaced duplicated socket-selection logic across the scratch wrappers with
   calls to the shared helper
3. updated `scripts/pg17_scratch_psql.sh` usage text so the default target is
   stated plainly instead of implying an automatic home-directory fallback
4. added `scripts/tests/test_resolve_scratch_socket_dir.py` coverage for:
   - explicit override wins
   - home-directory fallback is refused
   - missing preferred scratch socket fails loudly
5. added a tiny test-only escape hatch in the helper
   (`TQV_SCRATCH_TEST_ACCEPT_FILES=1`) so the regression test can model socket
   presence under the sandbox without binding real AF_UNIX sockets

## Validation

Passed:

- `scripts/tests/run.sh`
- `python3 scripts/tests/test_resolve_scratch_socket_dir.py`
- `bash -n scripts/resolve_scratch_socket_dir.sh scripts/pg17_scratch_psql.sh scripts/load_real_corpus_scratch.sh scripts/bench_sql_latency_scratch.sh scripts/bench_sql_latency_verified_scratch.sh scripts/bench_pgvector_sql_latency_scratch.sh scripts/bench_tqvector_sql_overhead_breakdown_scratch.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as the rest of this branch:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This packet changes the scratch measurement posture from "best-effort guess" to
"explicit target or fail":

1. scratch wrappers now default to the actual scratch socket path only
2. commands no longer silently drift onto `~/.pgrx`
3. if the caller really intends the home-directory cluster, they can say so
   explicitly through `TQV_PG_SOCKET_DIR` or `PGHOST`

This is operational guardrail work, but it is directly relevant to ADR030/task
15 because the branch is now relying on real-corpus proof as merge evidence.

## Next Slice

Return to product-facing landing work:

1. keep tightening the runtime/documentation surface around the now-default
   `PqFastScan` operating point
2. or close the remaining task-15 proof surfaces against the canonical
   real-corpus lanes using the hardened wrappers
