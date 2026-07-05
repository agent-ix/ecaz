# Artifacts — Task 58.1 / slice 003 (Audit 2 + 3)

- head SHA: `37390579c`
- branch: `task-58-1-floor-recovery`
- task bucket: `reviews/task-58/`
- packet: `006-task-58-1-audit-2-3/`
- lane / fixture: N/A — structural unsafe-fn-delegation absorption only,
  no workload run.
- storage format / rerank mode: N/A
- isolated one-index-per-table vs shared-table: N/A (compile-gate only)

## Commands

- block count:
  ```
  grep -c "unsafe {" src/am/ec_hnsw/build_parallel.rs
  ```
  pre-slice (after slice 002 + 002.1): **84**
  post-slice: **74**

- doc parity (must hold throughout):
  ```
  grep -cE "^[ \t]*(pub(\(.*\))?\s+)?unsafe fn" src/am/ec_hnsw/build_parallel.rs
  grep -c "/// # Safety" src/am/ec_hnsw/build_parallel.rs
  ```
  result: 15 / 15 ✓

- compile-gate:
  ```
  cargo check --no-default-features --features pg18 --lib
  ```
  status: passes (only unrelated pre-existing
  `unused import` warning in `src/am/ec_spire/update.rs:26`).

- clippy on touched file:
  ```
  cargo clippy --no-default-features --features pg18 --lib 2>&1 \
    | grep -c "build_parallel.rs"
  ```
  hits in `build_parallel.rs`: **0**

## Key result lines `request.md` cites

- 10 inner `unsafe { ... }` blocks removed across 8 distinct
  `unsafe fn` bodies.
- Cumulative `build_parallel.rs` reduction from the Task 50
  baseline of 112: **112 → 74** (-33.9%).
- §Exit floor of ≤78 hit with **4-block margin**.
- Task 58 plan §Exit target of ≤70 sits 4 blocks above; that
  residue lives inside the worker-loop / leader-begin bodies
  whose multi-call inner `unsafe { ... }` blocks each carry
  narrower SAFETY rationales than the outer-fn contract — kept
  per the reviewer's preserved discipline.

## Timestamp

2026-05-25 — slice 003 committed as `37390579c` on
`task-58-1-floor-recovery`.
