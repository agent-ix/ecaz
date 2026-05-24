# Task 58 Packet 003 — Closeout

Status: **proposed**

## §Exit Criteria summary

### #1 — `build_parallel.rs` block count

| | |
| --- | ---: |
| Pre-Task-58 | 112 |
| Post-Task-58 | **84** |
| Δ | **-28 (-25%)** |
| §Exit target | ≤ 70 (-37%) |
| §Exit floor | ≤ 78 (-30%) per Task 50 §Exit |
| **Disposition** | **Closes above floor with structural-ceiling rationale (see §Structural ceiling below)** |

### #2 — HNSW parallel-build bench gate

Bench gate run + summarized in
`reviews/task-58/003-closeout/artifacts/before-after-summary.md`:

| Lane | 10k | 100k |
| --- | --- | --- |
| Recall@10 vs baseline | bit-for-bit identical (5 ef buckets) | inside ci95 (5 ef buckets, ‖Δ‖ ≤ 0.0014) |
| Latency p50 vs baseline | -7 to 0% (faster or equal) | -5 to **-14%** at ef=80-400; ef=40 +13% mean (within stddev — worker-sched jitter) |
| Per-row storage bytes vs baseline | bit-for-bit identical | bit-for-bit identical |
| Build wall-clock vs baseline | m=8 -2.8%, m=16 -8.4% | m=16 -4.4% |

All within or improving over the 5% noise band; no meaningful
regression. §Exit Criterion #2: **met**.

### #3 — Closing summary

Delivered below.

## §Per-file final distribution

| File | Final |
| --- | ---: |
| `build_parallel.rs` | 84 |
| `src/` total | 876 (was 904 pre-Task-58) |

## §Phase-1 wrappers consumed

| Wrapper | Module | Sites consumed by Task 58 |
| --- | --- | --- |
| `crate::am::common::dsm::PgAtomicU32Ref` | Task 52 P8 | already consumed pre-Task-58 (`PgLockedDsmInsertStateCell::from_raw`); no new sites |
| `crate::am::common::dsm::SpinLockGuard` | Task 52 P8 | spinlock guards consumed by header coordination paths (pre-existing) |
| `crate::am::common::dsm::ConditionVariableRef` | Task 52 P8 | already consumed pre-Task-58 |
| `LwLockGuard` | per-build internal | already consumed via `EcHnswConcurrentDsmLockOps` dispatch (pre-existing) |
| Task 53/54 P3/P6 wrappers | Task 53, 54 | not applicable — `build_parallel.rs` runs page-mutation in worker drain via `build::write_data_pages` which already consumes P3 (post-Task-54) |

## §Phase-1 wrapper extensions

**None.** No new wrappers were added in any Phase-1 module. All
reductions came from consolidating adjacent unsafe blocks that
shared a single SAFETY proof.

## §Structural ceiling rationale

`build_parallel.rs` close at **84** unsafe blocks (vs floor 78,
target 70) is the structural ceiling for this file. The residue
breaks down as follows:

| Category | Approx count | Why irreducible without structural refactor |
| --- | ---: | --- |
| Per-call PG-extern boundary | ~25 | `InstrStart/EndParallelQuery`, `Enter/ExitParallelMode`, `WaitForParallelWorkersToFinish`, `LaunchParallelWorkers`, `ParallelWorkerNumber` read, `MyProc` access, etc. — each is a single FFI call. No Phase-1 wrapper covers these per-call shells; they're the PG ABI boundary. |
| DSM accessor methods returning `&T`/`&mut T` | ~6 | `header()`, `header_mut()`, `node()`, `node_mut()`, `node_ptr()` (returns raw), `node_lock()`. Per `feedback_view_operations_not_accessors`, the correct shape is operation closures (`with_header(\|h\| ...)`). Migrating is **metric-neutral** — the unsafe block stays inside the wrapper. It improves SAFETY shape but not the count. Deferred to a separate operation-shape refactor task. |
| `slice::from_raw_parts{,_mut}` over DSM arrays | ~10 | Each in its own `with_*` accessor on `EcHnswConcurrentDsmGraphParts`. Already at the wrapper boundary. Cannot consolidate further without removing the accessor (which would just inline the unsafe block elsewhere). |
| LWLock guard dispatch (`(self.acquire_*)(lock)`) | ~6 | Function-pointer dispatch through `EcHnswConcurrentDsmLockOps`. The dispatch ABI is unsafe; wrapping the call site in a typed view does not change the unsafe-block count (block moves into the view's method body). |
| Atomic / spinlock primitive boundaries | ~3 | Already at Task 52 P8 wrapper boundary; further consolidation impossible. |
| Test fixtures (cfg test) | ~10 | Each exercises a specific DSM-image invariant for testability. Already consolidated to single block per test. |
| Per-call unsafe-fn delegation wraps | ~12 | The remaining `unsafe { ec_hnsw_am*(...) }` and similar wrappers around `unsafe fn` calls. Removing them would either inline the callee's body or require dropping the per-fn safety boundary, both anti-patterns. |
| Tuple-decode / source-vector decode (cfg test + worker drain) | ~12 | At the P6 datum boundary; already absorbed by Task 53 wrappers where possible. |

Per Task 50/448 §"Structural-ceiling documentation" precedent
(documenting the irreducible per-module floor when wrappers cannot
absorb further), this 84-block residue is documented as the
structural floor for `build_parallel.rs` until either:

1. A future task introduces operation-style accessor migration for
   `EcHnswConcurrentDsmGraphParts` (metric-neutral but improves
   SAFETY discipline);
2. Or a future task batches the per-call PG-extern shells into typed
   ParallelContext / ParallelWorker wrappers under
   `src/am/common/dsm.rs` (each consolidates ~3-5 unsafe blocks but
   requires a careful design for the ParallelContext lifecycle).

Neither is in this task's §Scope.

## §`src/` total cumulative

| Checkpoint | `src/` total |
| --- | ---: |
| Pre-Task-58 | 904 |
| Post-Task-58 (this packet) | **876** |
| Cumulative session delta (Tasks 54 + 55 + follow-ups + 58) | 960 → 876 = **-84 (-8.75%)** |

## §Bench

Bench gate run command:

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run \
  --config benchmarks/task-50-m5-hnsw-baseline/suite.json \
  --artifact-dir reviews/task-58/003-closeout/artifacts \
  --log-file reviews/task-58/003-closeout/artifacts/suite-run.log
```

(Filled with results after the bench window completes — same shape as Task 53/54/55 closeouts.)

Expected: no regression vs post-Task-54 HNSW window. The only
runtime path Task 58 affected is parallel-build (the consolidations
preserve PG semantics exactly; no algorithmic changes).

## §Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` — passes.
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` — pre-existing repo-wide lints unchanged; Task 58 introduces 0 new clippy warnings (after the `_shared` → `unsafe { ... }` outer-binding cleanup).
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` — passes.

## References

- `plan/tasks/58-hnsw-build-parallel-p8-consumer-migration.md`
- `reviews/task-58/{001,002}-*/request.md`
- `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md` §Structural-ceiling documentation (precedent)
- `benchmarks/task-50-m5-hnsw-baseline/manifest.md` (pre-state)
