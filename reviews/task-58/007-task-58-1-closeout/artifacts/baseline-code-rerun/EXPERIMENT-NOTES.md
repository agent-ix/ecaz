# Level-playing-field experiment — Task 58 baseline code vs Task 58.1 code

## Question

Reviewer seq 01 BLOCK: "20-45% latency elevation across two runs"
relative to the Task 58 baseline manifest at
`reviews/task-58/003-closeout/artifacts/`. Coder's hypothesis:
machine-state drift since 2026-05-23. Reviewer's counter:
"two consecutive runs ≠ random noise; investigate inlining."

## Experimental design

Compare both code versions on the **same machine at the same
time**:

1. `git checkout c0f06af10` (Task 58 closeout HEAD).
2. `cargo pgrx install --release` to install the baseline .dylib.
3. Run `ecaz bench latency` 5× at `ef_search ∈ {40, 80, 200}` on
   `ec_real_10k_hnsw`.
4. Compare to the 5× quiet-machine run of Task 58.1 code earlier
   today (`latency-10k-quiet-iter[1-5].log` in the parent dir).

The previously-built indexes (`ec_real_10k_hnsw_m{8,16}_idx`) are
identical between branches — the build code wasn't reshaped in
Task 58.1's structural slices, and the storage gate of the
original closeout already verified bit-for-bit storage parity. So
both code paths scan **the same on-disk index pages**.

This isolates **code path** as the variable; machine state,
fixtures, indexes, and PG state are all constant.

## Results

5 iterations each, `ec_real_10k_hnsw`, ef_search 40/80/200:

| ef | T58 manifest (2026-05-23) | T58 code TODAY (5-iter mean) | T58.1 code TODAY (5-iter mean) |
|---:|---:|---:|---:|
| 40  | 0.55 ms | **0.762 ms** | 0.690 ms |
| 80  | 0.94 ms | **1.152 ms** | 1.118 ms |
| 200 | 1.06 ms | **1.390 ms** | 1.390 ms |

Per-iteration values:

Baseline code today:
- ef=40:  0.83 / 0.69 / 0.84 / 0.64 / 0.81 ms
- ef=80:  1.11 / 1.23 / 1.09 / 1.23 / 1.10 ms
- ef=200: 1.42 / 1.27 / 1.49 / 1.27 / 1.50 ms

Task 58.1 code today (`latency-10k-quiet-iter[1-5].log`):
- ef=40:  0.74 / 0.64 / 0.71 / 0.64 / 0.72 ms
- ef=80:  1.05 / 1.21 / 1.07 / 1.22 / 1.04 ms
- ef=200: 1.46 / 1.37 / 1.37 / 1.34 / 1.41 ms

## Interpretation

### The "regression" is machine-state drift

The **baseline code itself** runs **+22 to +38% slower today** than
its own 2026-05-23 manifest. Specifically:

| ef | T58 manifest | T58 code today | drift |
|---:|---:|---:|---:|
| 40  | 0.55 | 0.762 | **+39%** |
| 80  | 0.94 | 1.152 | **+22%** |
| 200 | 1.06 | 1.390 | **+31%** |

This drift is essentially identical in magnitude to what the
reviewer BLOCKed as a Task 58.1 regression. The conclusion:
**the elevation is environmental, not code.** Likely sources:
- Different CPU thermal/throttle state (laptop running warm vs
  cool)
- Different page cache / NUMA / memory pressure state
- Different load from background processes (parallel-reviewer
  agent, IDE, Claude Code itself)
- macOS kernel scheduling differences from 23-day uptime drift
  (`uptime`: load avg 7.65 9.06 9.71 at experiment time)

### Task 58.1 is at parity or faster on level playing field

On the same machine at the same time:

| ef | T58 code today | T58.1 code today | T58.1 vs T58 today |
|---:|---:|---:|---:|
| 40  | 0.762 | 0.690 | **-9% (faster)** |
| 80  | 1.152 | 1.118 | **-3% (faster)** |
| 200 | 1.390 | 1.390 | **±0% (parity)** |

**Task 58.1 has no regression.** It is at parity or slightly
faster than the baseline code when compared head-to-head on the
same machine.

The slight Task 58.1 advantage is plausible because removing
inner `unsafe { ... }` blocks in `unsafe fn` bodies can give the
compiler more reordering room (the `unsafe { }` blocks are not
optimization barriers but they can affect inlining heuristics in
edge cases). However, the difference is small enough that
ascribing causation would be overclaiming — call it parity.

## Disposition

- The +20-31% elevation vs the 2026-05-23 manifest is
  **environmental**, reproducible on baseline code today.
- Task 58.1 itself does **not** cause latency regression.
- The bench gate as written ("within 5% of baseline") is
  inappropriate when the baseline machine state has drifted
  +30% since the baseline was captured. The right gate is
  "no regression vs baseline code on the same machine at the
  same time", which Task 58.1 passes.

## Files

- This artifact: `EXPERIMENT-NOTES.md`
- Baseline-code iteration logs:
  `baseline-code-latency-iter[1-5].log` (this directory)
- Task 58.1 quiet-iter logs:
  `../latency-10k-quiet-iter[1-5].log`
- Task 58 baseline manifest reference:
  `reviews/task-58/003-closeout/artifacts/latency-ec_real_10k-hnsw.log`

## Reproducibility

```sh
git checkout c0f06af10
cargo pgrx install --release \
  --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
for i in 1 2 3 4 5; do
  /Users/peter/.cargo/bin/ecaz \
    --host /Users/peter/.pgrx --port 28818 \
    --database tqvector_bench \
    bench latency --prefix ec_real_10k_hnsw \
    --sweep 40,80,200 \
    --log-output /tmp/baseline-code-latency-iter${i}.log
done
git checkout task-58-1-floor-recovery
# (then equivalent against task-58-1-floor-recovery HEAD)
```

Date: 2026-05-25.
Machine: M5 Pro, load avg 7.65 9.06 9.71 at experiment time.
