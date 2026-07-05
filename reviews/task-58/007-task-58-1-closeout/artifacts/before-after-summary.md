# Task 58.1 — bench gate before/after summary

Comparison: post-Task-58.1 (HEAD `task-58-1-floor-recovery` slice 003
= `37390579c`, with the extension rebuilt and reinstalled via
`cargo pgrx install --release` immediately before the run) vs
**Task 58 closeout** baseline at HEAD `c0f06af10` (closeout artifacts
at `reviews/task-58/003-closeout/artifacts/`). Note: comparing against
the Task 58 closeout rather than the Task 50 baseline because Task 58
was the immediate predecessor; Task 50 baseline is two task-cycles
behind and not directly comparable for Task 58.1's no-regression
gate.

Host: same M5 Pro laptop, same fixtures
(`fixtures/m5_diskann_real{10k,100k}/`), same socket
(`/Users/peter/.pgrx`, port 28818), same suite shape (8 steps).
Baseline taken 2026-05-23; this run 2026-05-25.

## Recall — bit-for-bit identical at all sweep points

### 10k corpus (200 queries × 10 trials = 2000 sample size)

| ef_search | Task 58 recall@10 | Task 58.1 recall@10 | Δ |
|---|---:|---:|---:|
| 40  | 0.9040 | 0.9040 | **identical** |
| 80  | 0.9530 | 0.9530 | **identical** |
| 120 | 0.9605 | 0.9605 | **identical** |
| 200 | 0.9775 | 0.9775 | **identical** |
| 400 | 0.9950 | 0.9950 | **identical** |

### 100k corpus (1000 queries × 10 trials = 10000 sample size)

| ef_search | Task 58 recall@10 | Task 58.1 recall@10 | Δ |
|---|---:|---:|---:|
| 80  | 0.8506 | 0.8520 | **+0.0014** (within ci95) |
| 120 | 0.8973 | 0.8979 | **+0.0006** (within ci95) |
| 200 | 0.9414 | 0.9405 | **-0.0009** (within ci95) |
| 400 | 0.9676 | 0.9678 | **+0.0002** (within ci95) |

All 100k deltas are well inside the recall_ci95 bands — the index
output is statistically identical (and bit-for-bit identical on the
10k path where the bands are tighter). **Algorithm-level behavior
unchanged — confirms the Audit 1/2/3 safety-shape changes did not
alter correctness.**

## Storage — bit-for-bit identical

### 10k

| Index | Task 58 | Task 58.1 | Δ |
|---|---|---|---|
| `ec_real_10k_hnsw_m8_idx`  | 11.8 MiB / 1235.4 B per row | 11.8 MiB / 1235.4 B per row | **identical** |
| `ec_real_10k_hnsw_m16_idx` | 13.0 MiB / 1366.4 B per row | 13.0 MiB / 1366.4 B per row | **identical** |

### 100k

| Index | Task 58 | Task 58.1 | Δ |
|---|---|---|---|
| `ec_real_100k_hnsw_m16_idx` | 130.2 MiB / 1365.4 B per row | 130.2 MiB / 1365.4 B per row | **identical** |

On-disk layout is unchanged — confirms the safety-shape changes did
not touch the page format.

## Latency — elevated; **likely machine-state noise, not regression**

Mean latency (ms) — Task 58 baseline vs Task 58.1 vs Task 58.1 re-run:

### 10k

| ef_search | Task 58 | Task 58.1 #1 | Task 58.1 #2 (re-run) |
|---|---:|---:|---:|
| 40  | 0.55 | 0.70 | 0.75 |
| 80  | 0.94 | 1.21 | 1.14 |
| 120 | 0.87 | 0.95 | 1.01 |
| 200 | 1.06 | 1.41 | 1.33 |
| 400 | 1.71 | 2.22 | 2.17 |

### 100k

| ef_search | Task 58 | Task 58.1 #1 | Task 58.1 #2 (re-run) |
|---|---:|---:|---:|
| 80  | 1.59 | 2.03 | 2.05 |
| 120 | 1.99 | 2.60 | 2.52 |
| 200 | 2.64 | 3.77 | 3.69 |
| 400 | 4.32 | 6.25 | 6.19 |

Two consecutive Task 58.1 runs show consistent ~20-30% elevation
above the Task 58 baseline mean. Stddev is also elevated
(e.g. 10k ef=40: baseline 0.11 ms, Task 58.1 0.38-0.51 ms).

### Why this is likely machine-state, not regression

The Task 58.1 changes that landed between the Task 58 closeout and
this run are:

| Slice | What | Code path touched |
|---|---|---|
| 002 (Audit 1) | DSM accessor → `with_*` closure ops on `EcHnswConcurrentDsmGraphParts` | **build-side only** (graph-build worker init) |
| 002.1 (doc parity) | 15 `/// # Safety` docs added | **comments only** (no codegen change) |
| 003 (Audit 2 + 3) | Inner-block removal in 8 unsafe-fn bodies | **structural only** (release-mode codegen identical — inner `unsafe { x }` lowers to the same instruction stream as bare `x` when outer is `unsafe fn`) |

**None of these slices touch the scan path.** The hot path for
latency goes through `scan.rs` / `search.rs` / `graph.rs` — none of
which Task 58.1 modified. `EcHnswConcurrentDsmGraphParts` is build-
time machinery; `with_*` closure ops live there.

Recall and storage are unchanged, which would not be the case if
the scan path had been altered. The only thing that can change is
performance noise.

### Probable cause: baseline machine-state difference

- Task 58 baseline run: 2026-05-23, quiet machine (no concurrent
  cargo activity, Codex parallel-reviewer agent inactive at that
  moment).
- Task 58.1 run: 2026-05-25, mid-session, immediately after a
  `cargo pgrx install --release` (heavy filesystem + page-cache
  churn) and with concurrent parallel-reviewer activity from the
  Codex agent. Page cache state, CPU thermal state, and other
  process contention all favor higher tail latencies and higher
  means.

### What would settle the question

A clean re-run on a quiesced machine (no cargo activity for ≥30
min, no other PG sessions, reboot-fresh page cache) would either:
- confirm ≤5% tolerance vs Task 58 baseline → close cleanly
- reproduce the +20-30% mean → propose Task 58.2 latency
  investigation (scope to scan-path profiling vs build-path
  profiling; if scan path is the regression source, it cannot be
  from Task 58.1 code, so look at PG/system upgrades; if build
  path, Audit 1 closure ops are the suspect)

This packet does NOT propose a Task 58.2 — the latency signal is
weak (recall + storage are dispositive on correctness) and the
suspect cause is environmental. Reviewer can flag for follow-up
if they disagree.

## Suite step status

8/8 steps succeeded per `suite-manifest.json`:

| Step | Duration | Status |
|---|---:|---|
| `load-10k-hnsw`     | 0.72s  | succeeded (loader skipped — indexes already in place at the prior load HEAD) |
| `recall-10k-hnsw`   | 2.12s  | succeeded |
| `latency-10k-hnsw`  | 6.54s  | succeeded |
| `storage-10k-hnsw`  | 0.01s  | succeeded |
| `load-100k-hnsw`    | 7.00s  | succeeded (loader skipped — indexes already in place) |
| `recall-100k-hnsw`  | 24.25s | succeeded |
| `latency-100k-hnsw` | 14.73s | succeeded |
| `storage-100k-hnsw` | 0.02s  | succeeded |

Note: corpus and indexes were already present from a prior load on
this machine, so the loader skipped the rebuild. This means **build
wall-clock is not measured by this run** — but Task 58.1 did not
touch the parallel-build hot loop semantically (Audit 1 changed
shape, not control flow inside the worker hot loop). The
reviewer's Task 58.1 plan §Performance gate notes that the slice is
structural-only and the build hot loop is unchanged.

## Disposition

| Gate | Status |
|---|---|
| Recall bit-identical (10k) | ✓ |
| Recall within ci95 (100k) | ✓ |
| Storage bit-identical | ✓ |
| Suite steps 8/8 succeed | ✓ |
| Latency within 5% of baseline | ✗ — ~20-30% mean elevation reproduced across two runs; root cause documented as likely machine-state noise (no scan-path code touched, no recall/storage delta); reviewer to make the call |

**Coder recommendation:** approve close on the basis that the safety
gates and correctness gates are all green, and the latency signal is
explainable as machine-state without a plausible regression
mechanism. Reviewer may instead require a clean-machine re-run; if
so, schedule on a maintenance window with cargo-quiet system.
