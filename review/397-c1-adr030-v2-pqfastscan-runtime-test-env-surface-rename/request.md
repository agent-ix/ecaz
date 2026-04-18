# Review Request: C1 ADR-030 V2 PqFastScan Runtime Test Env Surface Rename

## Context

Packets 392, 394, 395, and 396 moved the runtime/debug surface toward
canonical `PqFastScan` naming and kept the old ADR030 env names only as
compatibility fallbacks.

But several direct runtime tests in `src/lib.rs` still explicitly set:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_MODE`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE`
- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY`

That left the regression surface itself lagging behind the canonical runtime
surface it was supposed to validate.

## Problem

Without this slice, the branch still demonstrates the old ADR030 env names in
normal runtime tests even though:

- the canonical env names already exist
- the canonical debug helpers already exist
- the old ADR030 env names are now only compatibility fallbacks

That is the wrong direction for a `main` landing branch.

## Planned Slice

One test-only cleanup checkpoint:

1. move the remaining direct `PqFastScan` runtime tests onto canonical
   `TQVECTOR_PQ_FASTSCAN_*` env names
2. leave the runtime fallback logic intact
3. avoid any AM behavior change

## Implementation

Updated:

- `src/lib.rs`

Switched the remaining direct runtime tests from the legacy ADR030 env names to
canonical `PqFastScan` names:

- `TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE`
- `TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW`
- `TQVECTOR_PQ_FASTSCAN_RERANK_MODE`
- `TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL`
- `TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT`
- `TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE`
- `TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY`

This affects only the regression surface. The compatibility fallback logic in
the runtime settings helper remains unchanged.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This is a regression-surface cleanup checkpoint:

1. the remaining direct runtime tests now demonstrate canonical
   `TQVECTOR_PQ_FASTSCAN_*` env usage
2. the old ADR030 env names remain only in compatibility fallback code
3. no runtime behavior changed

What this slice intentionally does **not** do:

- remove the ADR030 compatibility fallbacks
- rename internal grouped counter variable names in older tests
- change any scan, rerank, or exact-traversal semantics

## Next Slice

The main remaining work is no longer naming cleanup. The next meaningful slices
are closer to landing proof and final branch convergence:

1. tighten any remaining parity proof or migration proof gaps from task 15
2. review whether any non-test compatibility aliases should be removed before
   merge or left for follow-up
