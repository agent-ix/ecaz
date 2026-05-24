---
task: 50
packet: 236
topic: planner-cost-global-accessors
role: coder
status: ready-for-review
created: 2026-05-21T05:31:48-07:00
head_sha: a05e664540ce31a5f0728f2ab3214524bd6fedcb
---

# Review Request: Planner Cost Global Accessors

## Summary

This packet removes repeated caller-side unsafe blocks around PostgreSQL planner cost global reads.

Changes:

- Made `current_planner_cost_constants` safe while keeping the PostgreSQL global reads inside the common accessor.
- Made `current_cpu_tuple_cost` safe for the same reason.
- Removed repeated unsafe blocks from IVF, SPIRE, SPIRE custom scan, DiskANN, HNSW, and common HNSW cost call sites.
- Added short accessor documentation clarifying that these functions read backend-local planner globals by value and do not impose a memory-safety precondition on callers.

## Safety Notes

- The actual PostgreSQL global reads remain isolated in `src/am/common/cost.rs`.
- Call sites still execute in planner/diagnostic contexts as before; this change only stops treating the read-only global access as a caller-owned unsafe boundary.
- Raw relation descriptor reads are not changed in this packet.

## Unsafe Count

- `src/am/common/cost.rs`: `15 -> 12`
- `src/am/ec_diskann/cost.rs`: `9 -> 7`
- `src/am/ec_hnsw/shared.rs`: `64 -> 63`
- `src/am/ec_ivf/cost.rs`: `10 -> 8`
- `src/am/ec_spire/cost/mod.rs`: `26 -> 24`
- `src/am/ec_spire/custom_scan/cost_helpers.rs`: `26 -> 24`
- Previous repo count: `2492`
- Current repo count: `2480`
- Delta: `-12`

The packet-local count log is:

- `artifacts/unsafe-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: scoped `rustfmt --check` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-cost-pg18-no-run.log`: `cargo test --lib cost --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
