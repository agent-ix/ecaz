# Artifact Manifest

- Source HEAD: `2cb56a6f2`
- Task bucket: `reviews/task-38/`
- Packet: `reviews/task-38/006-interrupt-boundary-inventory/`
- Capture date: `2026-07-26 America/Los_Angeles`
- Host: Apple M5, macOS arm64
- Lane: source inventory and follow-up definition
- Fixture/storage/rerank/benchmark mode: not applicable
- Runtime isolation: no PG fixture or runtime lane executed

## `explicit-interrupt-sites.log`

- Commands:
  - `rg -n "CHECK_FOR_INTERRUPTS|check_for_interrupts|ProcessInterrupts|maybe_check_for_interrupts" src/am/ec_hnsw src/am/ec_ivf src/am/ec_diskann src/am/ec_spire src/am/ec_distann -g '*.rs' | sort`
  - `rg -n "PostgresInterruptPoll|postgres_query_cancel_pending|postgres_statement_timeout_pending|InterruptPending|QueryCancelPending" src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`
- Result: exact current explicit poll-site source listing. Import lines and
  safety comments remain in the raw result so a reviewer can distinguish them
  from executable call sites.

## Static Validation

- `git diff --check`: pass.
- Tests: not run; documentation/task-definition-only checkpoint.
- Architecture-specific execution: not applicable and not claimed.

