# Task 65: DiskANN Build Performance — Vamana Core Overhaul

Status: **complete** 2026-05-28. Direct follow-up to Task 29c
(`plan/tasks/29c-diskann-build-perf.md`) and packet `11104`'s
pass-1 Vamana timing split. Single-process build only; parallel
graph construction is out of scope (deferred to Task 65b,
`plan/tasks/65b-diskann-build-parallel-graph-construction.md`).

Owner: coder. One coder, one branch, no fan-out.

Completion evidence:

- Code packets:
  `reviews/task-65/001-vamana-core-build-perf/request.md`
  and reviewer feedback in the same packet.
- Measurement packet:
  `reviews/task-65/002-vamana-core-measurement/request.md`
  and `reviews/task-65/002-vamana-core-measurement/artifacts/manifest.md`.
- Final code and packet closure commits:
  `8e860324c` (`Add Vamana build dhat profiler`) and
  `8e3109c25` (`Update DiskANN Vamana measurement closure evidence`).
- Final real10k build evidence: R32/L200 fixed-loader release build
  `14.92s` vs the `16s` target.
- Final recall evidence: real10k R32/L200 `0.9975 / 0.9975 / 0.9975`
  at L=`64,128,200`; synth10k R32/L200 `0.1610 / 0.2625 / 0.3270`
  at L=`64,200,800`, including L=200 within 0.5pp of the Task 29
  smoke baseline.
- Memory evidence: `dhat_vamana_build` first-1k real10k smoke plus
  static hot-loop audit under the measurement packet artifacts.

Investigation source: `plan/design/diskann-build-performance.md`.
Read that document first — it carries the line-by-line evidence
and complexity model behind every item below.

## Why

Task 29c stopped at a release-mode single-process build of 70.678 s
on real-10k vs HNSW's 5.23 s on the same corpus. The remaining gap
is **Vamana graph construction**, not heap scan, training, payload
encode, or page writes. Packet `11104` localised the cost to
pass-1 greedy search, robust-prune, and backlink repair.

The investigation in `plan/design/diskann-build-performance.md`
identifies six independent algorithmic / structural issues in the
Vamana core whose combined effect is quadratic in N. Even a clean
single-threaded fix should produce a multi-× speedup before any
parallel-build work is considered. This task lands those six fixes.

This is **not** a measurement-first task — the root causes are
already identified by code review. It is an implementation task
with a measurement gate at the end.

## Scope — the six issues

All six target `src/am/ec_diskann/{ambuild,vamana,build}.rs`.

### 1. Remove O(N²) exact-match heap-tuple dedup

`src/am/ec_diskann/ambuild.rs:116-147` (`BuildState::push`).

The current path linearly scans every previously-collected source
vector and bit-compares floats to collapse exact duplicates into an
`overflow_heap_tids` list. At N=100k×d=1536 this is ≈7.7×10¹³
float compares during the heap scan callback.

**Required change:** delete the dedup branch. Treat every heap TID
as its own node. Drop the `overflow_heap_tids` field on
`RawHeapTuple` and delete the per-tuple call to
`insert::stage_overflow_heap_tids_in_chain` in
`flush_build_state` (`ambuild.rs:385-394`). The runtime overflow
path (`insert::stage_overflow_heap_tids_in_chain`) stays — it is
still reachable from the per-row insert path; only the build-time
loop goes away.

If a corpus *does* contain exact-duplicate vectors, the graph will
contain duplicate nodes. That is correct behaviour for V1; document
it explicitly in the ambuild module docstring and in CHANGELOG.

### 2. Eliminate per-search `vec![false; n]` allocations in `greedy_search`

`src/am/ec_diskann/vamana.rs:212-213`.

Replace the two N-sized `Vec<bool>` allocations with a reusable
bitset structure threaded into the build orchestrator. Required
properties:

- Backed by `Vec<u64>` (one bit per node) — 8× less memory traffic
  than `Vec<bool>`.
- Owned by the build driver (`build_vamana_graph_with_pass1_extra_candidates`),
  cleared with `.fill(0)` between calls.
- `greedy_search` takes `&mut SearchScratch` instead of allocating
  internally.
- Public `greedy_search` API may keep its current signature as a
  thin allocate-and-call wrapper for callers outside the build
  driver (e.g. tests). Inside the build, every call goes through
  the scratch-taking variant.

Acceptance: zero `vec![false; n]` allocations during the pivot loop.
Verified by reading `cargo +nightly rustc -- -Zprint-mono-items`
output for the build path, or by a heaptrack-style smoke run on the
m5 fixture during the validation gate.

### 3. Replace linear-scan frontier with a bounded min-heap

`src/am/ec_diskann/vamana.rs:223-261` (the body of `greedy_search`).

Current:
```rust
let next = frontier.iter().copied().filter(|c| !visited_flag[..]).min_by(...);
...
if frontier.len() > list_size {
    frontier.sort();
    frontier.truncate(list_size);
}
```

Replace with a structure that supports:
- O(log L) push of a new candidate.
- O(log L) pop of the closest un-expanded candidate.
- O(1) check "is the current best distance worse than the L-th best
  seen so far?" — the early-stop condition Vamana uses to terminate
  search.

Reference implementation: pgvectorscale
`pgvectorscale/src/access_method/graph/mod.rs:75` uses
`BinaryHeap<Reverse<ListSearchNeighbor>>` for the un-expanded
frontier and a separate distance-ordered `Vec<ListSearchNeighbor>`
for the visited set. Port that shape directly — do not invent a
new data structure.

The visited bitset from item #2 is independent and stays.

### 4. Drop per-pivot bitsets in `candidate_pool_for_prune` and `append_candidates_for_prune`

`src/am/ec_diskann/vamana.rs:328-390`.

Both helpers allocate `vec![false; node_count]` to dedupe a
candidate set that is only `L + R` items wide. Same O(N²) churn
pattern as #2.

Options (pick the simplest that fits with #2's scratch struct):

- **Preferred:** share the bitset from #2's `SearchScratch`. The
  pool helpers already run inside the same pivot iteration as the
  greedy search that just used the scratch — clear once, reuse.
- **Acceptable alternative:** sort-then-dedup on the candidate
  vector, or a 64-entry small linear scan. Either is fine because
  the candidate count is bounded.

Acceptance: zero N-sized allocations inside the per-pivot loop.

### 5. Replace two-pass build with a single growing-α pass

`src/am/ec_diskann/vamana.rs:509-643` (the outer
`for (pass_index, &alpha) in [1.0f32, alpha_final].iter().enumerate()` loop).

Today the entire N-pivot loop runs twice — once at α=1.0, then
again at α=alpha_final. pgvectorscale instead runs **one** N-pivot
loop and grows α inside `robust_prune` until enough neighbors are
kept (`graph/mod.rs:413-484`):

```rust
let mut alpha = 1.0;
while alpha <= max_alpha && results.len() < num_neighbors {
    // prune at current alpha, then bump
    alpha *= 1.2;
}
```

Required change: port the growing-α loop into `robust_prune`,
delete the outer two-pass driver, and confirm that the
`pass1_extra_candidates` extension point (currently used for HNSW
warm-start; see
`build_vamana_graph_with_pass1_extra_candidates`) still has
somewhere to land — fold it into the single-pass driver as a
per-pivot "extra candidates" parameter applied on the first
robust-prune iteration.

Affects deterministic-seed tests:
`build.rs:bo_007_deterministic_for_fixed_seed` and any recall
golden expectations. New golden values are acceptable provided
recall on the existing fixtures stays at or above current numbers.

### 6. Parallelise the per-node codec encode step with rayon

`src/am/ec_diskann/ambuild.rs:312-325` (the `payloads: Vec<NodePayload>`
construction in `flush_build_state`).

This is the easiest parallelism win: each `codec.encode(&t.source_vector)`
call is pure and independent. Convert the `.iter().map(...).collect()`
to a rayon `par_iter().map(...).collect()` using the existing
rayon dependency (HNSW build already uses it; see Task 26 / Task 58
for the consumer pattern).

Out of scope here:
- Parallelising the **graph construction** loop. Pivot insertion
  depends on prior graph state and the safe parallel pattern needs
  batching + a neighbor cache (pgvectorscale's approach). That is
  the Task 65b shape, not this task.
- Postgres `ParallelContext` workers. Same reason.

Acceptance: encode step shows near-linear scaling with rayon
threads on the m5 fixture (e.g. ≥3× on 4 cores).

## Non-goals

- **No SIMD changes.** Distance kernels stay as they are.
- **No persistence layer changes.** `persist_vamana_graph`'s
  placeholder-then-patch design stays.
- **No diagnostic counter gating** (`Cell`-based counters from
  P1 #7 in the investigation doc). That's a separate cleanup; if
  the coder wants to land it as part of this task because it's
  trivially in the way, that's fine, but it is not required.
- **No parallel graph construction.** See #6 above.
- **No changes to the runtime insert / scan path.** Build only.

## Validation gate

This task is implementation-first but the gate is measured.

1. **Functional gate** — all existing `ec_diskann` tests must pass:
   - `cargo test -p ecaz --features pg18 ec_diskann`
   - `cargo pgrx test pg18` for the DiskANN integration tests.
   - Existing recall fixtures (`fixtures/m5_diskann_real10k/`,
     `fixtures/m5_diskann_synth10k/`) must hold or improve on
     current recall numbers. Determinism goldens may be updated.
2. **Performance gate** — release-mode build of real-10k must drop
   substantially from the 70.678 s Task 29c floor. Concrete target:
   - Either match HNSW's 5.23 s within 3× (i.e. ≤ 16 s), OR
   - Show that the remaining cost is in a phase explicitly out of
     scope for this task (e.g. persist) and that the in-scope
     phases (greedy + robust-prune + backlink + encode) collectively
     account for ≤ 5 s on real-10k.
3. **Memory gate** — heaptrack / `dhat` smoke run on the m5 fixture
   shows no `vec![false; n]` allocation in the build hot loop.
4. **Behavioural gate** — packet must include a recall delta on
   real-10k and synth-10k vs the Task 29c floor. Recall must hold
   within 0.5 percentage points of current numbers.

## Slice plan

Each slice is its own commit and review packet. Keep them narrow.

- **Slice A (#1, #5 setup):** delete heap-tuple dedup; collapse the
  two-pass loop to a single-pass shell that still calls the old
  α-static `robust_prune`. Recall determinism golden updates land
  here.
- **Slice B (#5 finish):** port pgvectorscale's growing-α
  `robust_prune` loop. Wire `pass1_extra_candidates` into the
  single-pass driver.
- **Slice C (#2, #4):** introduce `SearchScratch` bitset, thread it
  through `greedy_search`, `candidate_pool_for_prune`, and
  `append_candidates_for_prune`. Public allocate-and-call wrappers
  for non-build callers.
- **Slice D (#3):** replace the frontier with the bounded min-heap
  + separate visited list.
- **Slice E (#6):** rayon-parallelise codec encode.
- **Slice F:** measurement packet. Release-mode m5 fixture run,
  recall checks, heaptrack snapshot, before/after timing split.

A single packet per slice under
`reviews/task-65/{NNN}-{slug}/request.md` per CLAUDE.md packet
rules.

## Coder workflow notes

- **Push on every slice.** Slices C/D in particular interact with
  in-flight refactor work elsewhere on the branch; visibility
  matters.
- **Skip tests by default** per CLAUDE.md "do not run tests by
  default" rule, except at slice boundaries where the change is
  algorithmic enough that static review is not sufficient — Slices
  B, C, D, and F all warrant `cargo test ec_diskann` at minimum
  before the slice commit lands.
- **No pgrx-test on macOS** for callback-touching slices because of
  the `_BufferBlocks` blocker
  (memory `feedback_dyld_buffer_blocks_known`). Validate Slice F
  on Linux or via the ecaz CLI corpus path.
- **No `/tmp` benchmark logs** — all timing evidence goes into the
  packet's `artifacts/` directory per CLAUDE.md `NFR-007`.

## Out of scope (re-stated)

- Parallel graph construction (deferred to Task 65b).
- Postgres `ParallelContext` integration.
- Persist-layer rewrite (double-clone in `persist_vamana_graph` —
  separate task if it shows up in Slice F numbers).
- SIMD distance kernels.
- Build-stats `Cell` counter gating (optional cleanup, not
  required).
- Anything in `src/am/ec_hnsw/`, `src/am/ec_ivf/`,
  `src/am/ec_spire/`.

## Acceptance criteria

1. All six listed issues are addressed by code changes on the task
   branch, each in its own commit + packet.
2. Validation gate (functional + performance + memory + behavioural)
   passes on the real-10k and synth-10k fixtures.
3. A summary packet documents the before/after timing split, recall
   delta, and a brief note on whether the next-largest cost is in a
   future-task lane (persist, parallel build, SIMD).
4. No regressions in DiskANN insert or scan tests.

## Estimated size

Medium. The six issues are each small in line count (≈ 200–400
LOC each), but #3 + #5 together change the algorithmic shape of
greedy_search and need careful determinism-test updates. Expect
1–2 weeks for a single coder with the validation gate.

## References

- Investigation source:
  `plan/design/diskann-build-performance.md`
- Build algorithm design:
  `plan/design/diskann-build-algorithm.md`
- Prior tuning task: `plan/tasks/29c-diskann-build-perf.md`
- Parallel build (HNSW): `plan/tasks/26-parallel-index-build.md`,
  `plan/tasks/58-hnsw-build-parallel-p8-consumer-migration.md`
- pgvectorscale reference (read-only): clone the upstream
  `timescale/pgvectorscale` repo and consult
  `pgvectorscale/src/access_method/graph/mod.rs` and
  `pgvectorscale/src/access_method/build.rs`.
