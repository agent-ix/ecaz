# 30717 — SPIRE libpq global dispatch governance

Code commit: `6e0186a826a324e68ec1034f662fc7cb89a72a2d`

This packet lands the first Phase 11 Stage C cross-query governance slice for
the remote-search libpq executor. It does not claim async execution or
cancellation; it adds nonblocking admission/backpressure around actual libpq
remote work so concurrent coordinator backends can be bounded before opening
remote sockets.

## What Changed

- Added session GUCs:
  - `ec_spire.remote_search_max_concurrent_dispatches`
  - `ec_spire.remote_search_max_concurrent_dispatches_per_node`
- Added nonblocking advisory-lock admission around compact candidate and remote
  heap libpq dispatches.
- Saturated global or per-node governance slots return
  `remote_executor_overload` with `next_blocker = remote_executor_governance`
  before conninfo secret lookup, socket open, remote index resolution, or
  endpoint identity probing.
- Extended `ec_spire_remote_search_libpq_executor_budget_summary(...)` to
  report the active concurrency caps alongside the existing per-query fanout and
  timeout caps.
- Updated `plan/design/spire-libpq-executor-budget.md` and the Phase 11 Stage C
  task plan to record the first cross-query governance surface.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_libpq_executor_global_governance_overload`
- `cargo pgrx test pg18 test_ec_spire_libpq_executor_budget_limits`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `git diff --check`

## Review Focus

- Confirm advisory-lock slot admission is acceptable as the first local
  cross-backend governance primitive before the async/pipeline executor lands.
- Confirm saturated governance should share `remote_executor_overload` status
  but use `remote_executor_governance` as the runtime blocker, distinct from
  pre-dispatch per-query `remote_executor_budget`.
- Confirm the remaining Stage C split is still right: cancellation propagation
  and true overlapped libpq execution remain follow-up slices.
