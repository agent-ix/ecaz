# Task 145 Packet 003: Remote Rerank Width

## Request

Review code commit `4d7e927f0` for the Task 145 packet 001 feedback item:
the remote coordinator rescore path now honors effective `rerank_width`.

This is a narrow code checkpoint. It does not claim Task 145 closeout; the
remote `ecaz bench suite` `remote:true` release A/B and the remaining economy
phases are still owed.

## Changes

- Compact remote candidates are sorted with the existing remote-candidate
  comparator and truncated to the effective rerank width before exact heap
  rescore.
- Width `0` preserves the prior full-frontier behavior.
- Production remote candidate/heap request state propagates coordinator
  `scan_plan.rerank_width`.
- Production libpq heap receive sets remote session `ec_spire.rerank_width`
  via `set_config` before heap receive SQL.
- Focused tests cover full-frontier width `0`, positive-width truncation, and
  production request propagation.

## Validation

Packet-local manifest: `artifacts/manifest.md`

- `cargo test remote_heap_rerank_prefix --no-default-features --features pg18`
- `cargo test production_executor_compact_receive_requests_use_dispatch_state --no-default-features --features pg18`
- `cargo test production_executor_heap_receive_requests_carry_tuple_payload_columns --no-default-features --features pg18`

All three validations passed.

