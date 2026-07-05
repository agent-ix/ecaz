# Review Request: C1 ADR-030 V2 Scratch Socket Override Visibility

Current head: `6b4eac0`

## Context

Packet `402` hardened the scratch wrappers so they no longer silently fall back
from `/tmp/tqvector_pgrx_home` to `~/.pgrx`.

Reviewer feedback on packets `401` and `402` then called out three remaining
operational gaps:

1. `PGHOST` still overrode the preferred scratch socket invisibly
2. the helper tests did not pin the `PGHOST` precedence branch
3. the wrappers had only helper-level coverage, not one end-to-end smoke
   invocation through a representative launcher

There was also one documentation-level drift note from `401`:

4. `scripts/restart_adr030_scratch.sh` duplicated the runtime default values
   without any pointer back to the Rust constants they mirror

## Problem

The scratch-targeting policy was correct but still slightly opaque in the one
case that matters most during measurement work:

- if `PGHOST` was set in the shell and the preferred scratch socket also
  existed, the helper would quietly honor `PGHOST`
- that is legitimate precedence, but it leaves the operator no clue that the
  wrapper is bypassing the default scratch cluster

The regression suite also stopped just short of the branch that caused concern:

- helper tests did not cover `PGHOST`
- no wrapper smoke test proved that one real launcher actually threads the
  resolved socket into `psql`

## Planned Slice

One narrow scripts-only follow-up:

1. keep the existing precedence contract
2. emit a stderr warning when `PGHOST` overrides an available preferred scratch
   socket
3. add hermetic regression coverage for that branch
4. add one representative `scripts/pg17_scratch_psql.sh` smoke test using a
   fake `psql` binary
5. leave a small comment in `restart_adr030_scratch.sh` tying its defaults to
   the Rust runtime constants they mirror

No AM/runtime behavior change.

## Implementation

Updated:

- `scripts/resolve_scratch_socket_dir.sh`
- `scripts/restart_adr030_scratch.sh`
- `scripts/tests/test_resolve_scratch_socket_dir.py`
- `scripts/tests/test_pg17_scratch_psql_socket_resolution.py`

Concrete changes:

1. `scripts/resolve_scratch_socket_dir.sh`
   - still resolves in the same order:
     - `TQV_PG_SOCKET_DIR`
     - `PGHOST`
     - preferred scratch socket
     - fail
   - now warns on stderr when `PGHOST` wins while the preferred scratch socket
     is actually present
   - now accepts test-only directory overrides through:
     - `TQV_SCRATCH_TEST_PREFERRED_SOCKET_DIR`
     - `TQV_SCRATCH_TEST_FALLBACK_SOCKET_DIR`
     so the policy can be tested hermetically without touching the real
     `/tmp/tqvector_pgrx_home`
2. `scripts/tests/test_resolve_scratch_socket_dir.py`
   - moved fully onto tempdir-backed preferred/fallback socket fixtures
   - added explicit coverage for the `PGHOST` precedence branch and warning
3. added `scripts/tests/test_pg17_scratch_psql_socket_resolution.py`
   - drives `scripts/pg17_scratch_psql.sh` end-to-end against a fake `psql`
     binary
   - proves the wrapper uses the helper-resolved preferred socket by default
   - proves an explicit `TQV_PG_SOCKET_DIR` override is threaded to `psql`
4. `scripts/restart_adr030_scratch.sh`
   - now carries an inline comment pointing at the Rust defaults it mirrors:
     - `PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW`
     - `PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME`

## Validation

Passed:

- `scripts/tests/run.sh`
- `bash -n scripts/resolve_scratch_socket_dir.sh scripts/pg17_scratch_psql.sh scripts/restart_adr030_scratch.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as the rest of this branch:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This packet responds directly to the reviewer nits without reopening the wider
scratch-targeting design:

1. `PGHOST` still wins when set, but it is no longer silent when it bypasses an
   available preferred scratch socket
2. the helper regression suite now proves that precedence branch explicitly
3. one real wrapper now has smoke coverage through a fake `psql`
4. the scratch restart defaults now make their dependency on Rust-side
   `PqFastScan` defaults visible

## Next Slice

Keep the remaining work on merge evidence, not tooling:

1. continue capturing current-head real-corpus proof on the canonical explicit
   `turboquant` / `pq_fastscan` index families
2. or address any new reviewer feedback that lands on packets `403+`
