# Task 58 Packet 002 — `build_parallel.rs` Adjacent-Unsafe-Block Consolidation

Status: **proposed**

First migration pass: consolidates adjacent unsafe blocks where they
share a single SAFETY proof. No behavioral or signature changes;
no Phase-1 wrapper extensions; preserves all SAFETY comments by
elevating each consolidated block to a fn-level SAFETY note.

## Per-file unsafe block count

| File | Pre | Post | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | **84** | **-28 (-25%)** |
| `src/` total | 904 | 876 | -28 |

§Exit floor (-30%, ≤78) not yet met. Remaining 6 reductions to floor
must come from operation-style accessor migration (§Slice plan 002+)
or accepting a structural-ceiling rationale for the per-call PG-extern
boundary residue.

## Consolidations applied

1. **`concurrent_dsm_graph_parts` constructor** (L1100s): 5 adjacent
   `unsafe { base.add(offset).cast() }` blocks for `header`, `nodes`,
   `neighbor_slots`, `codes`, `sources` → 1 outer `unsafe { ... }`
   covering all five.
2. **`complete_concurrent_dsm_graph_node_insert`** (L1418-L1422): 3
   adjacent `unsafe { nodes.add(idx) }` + `unsafe { addr_of_mut!((*node).lock) }`
   + `unsafe { locks.exclusive(lock) }` → 1 outer block returning
   `(node, _lock_guard)`.
3. **`EcHnswParallelGraphBuildLeader::begin`** parallel-mode setup: 8
   adjacent `unsafe { ... }` blocks (`shm_toc_allocate`, `ptr::write`,
   `ConditionVariableInit`, `SpinLockInit`, `shm_toc_insert`,
   `LWLockNewTrancheId`, `LWLockRegisterTranche`,
   `initialize_concurrent_dsm_graph_image`) → 3 consolidated blocks
   covering the InitializeParallelDSM-fail-path + DSM allocation +
   shared/graph image initialization.
4. **`EcHnswParallelBuildLeader::begin`** parallel-mode setup: same
   pattern as the graph-build leader; 5 adjacent unsafe blocks
   consolidated to 2.
5. **`EcHnswParallelGraphBuildLeader::wait_for_workers`** (L2335-L2353):
   3 adjacent blocks (`WaitForParallelWorkersToFinish`,
   `nworkers_launched` read, per-worker `InstrAccumParallelQuery`) → 1
   block covering the full finish sequence.
6. **`EcHnswParallelBuildLeader::finish`** (L2629-L2645): same pattern
   as the graph-build leader's finish sequence.
7. **`parallel_build_worker_main`** (L2720-L2740): `shm_mq_set_sender`
   + `shm_mq_attach` consolidated; `table_beginscan_parallel` +
   `table_index_build_scan` consolidated.

## Categorization of remaining 84

Most of the remaining unsafe is either:

- **Per-call PG-extern boundary** (`InstrStartParallelQuery`,
  `InstrEndParallelQuery`, `EnterParallelMode`, `ExitParallelMode`,
  `WaitForParallelWorkersToAttach`, etc.) — each a single FFI call;
  no Phase-1 wrapper covers these.
- **Accessor methods** (`header()`, `header_mut()`, `node()`,
  `node_mut()`, etc.) — returns `&T` / `&mut T` from `*mut T`.
  Migrating to operation closures (`with_header(|h| ...)`) does
  **not** reduce the unsafe count (the block stays inside the
  wrapper); it improves SAFETY shape per
  `feedback_view_operations_not_accessors` but is metric-neutral.
- **Atomic / spinlock dispatch** — already at the Task 52 P8
  wrapper boundary.
- **Test fixtures** (~10 blocks in `#[cfg(test)]` code) — each
  exercises a specific DSM-image invariant; consolidation already
  applied where possible.

## Validation

- `cargo check --no-default-features --features pg18 --lib` — passes.
- `cargo check --all-targets --no-default-features --features pg18,bench` — pending re-run after this commit.
- No behavior change: every consolidation preserves the exact
  semantics of the original code. SAFETY comments are elevated to
  cover the consolidated block.

## Files touched

- `src/am/ec_hnsw/build_parallel.rs`

## Next steps

- Packet 003: bench gate (HNSW build wall-clock at
  `parallel_workers ∈ {0, 2, 4}`) + closeout.
- Document structural-ceiling rationale for residue if final count
  stays above the §Exit-floor 78. Realistic close: ≤ 84 with explicit
  structural-ceiling rationale citing Task 50/448 precedent.
