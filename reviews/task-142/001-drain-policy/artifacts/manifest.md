# Task 142 packet 001 — scratch boundary-drain removal A/B: manifest

- Code under review: `86c05031d` ("Drop boundary drains from the IVF posting
  scratches", branch `task-141-sdot-kernel`, stacked on the Task 141 SDOT
  commit — irrelevant to this A/B: both cells run the default LUT scorer, so
  the int8 kernel is not exercised).
- Task bucket / packet: `reviews/task-142/001-drain-policy/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port 28818,
  db `tqvector_bench`. 2026-07-02.
- A/B form: before/after commit, same session, same tables, one dylib swap
  between cells (verified in-suite):
  - baseline `artifacts/baseline/`: `ecaz_build_git_sha()` = `2d98ec5b7...`
    (boundary drains present);
  - drainfix `artifacts/drainfix/`: `86c05031d...`.
- Fixture: the Task 135 packet 002 DENSE tables
  (`task135ab_dense_real{10k,50k,100k}`, `dense_posting_blocks=1`, plain
  turboquant, dbpedia 1536-dim). Default GUCs (LUT scorer, coalescing on).
  nprobe [32], k 10, warm, concurrency 1.
- Runner per cell at its sha:

  ```sh
  target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-142/001-drain-policy/task142-drain-ab-suite.json \
    --artifact-dir reviews/task-142/001-drain-policy/artifacts/<baseline|drainfix>
  ```

- Both runs exit 0, 10/10 steps.

## Key result lines

### Recall@10 — byte-identical

| scale | pre | post |
|---|---|---|
| 10k | 0.9734 | 0.9734 |
| 50k | 0.9521 | 0.9521 |
| 100k | 0.8969 | 0.8969 |

### Flush structure at 100k (the structural gate) — fully restored

| metric | pre (boundary drains) | post (capacity-only) | row-path reference (135/002) |
|---|---|---|---|
| kernel flushes / sweep | 1781 | **1311** | 1311 |
| width<32 flushes | 93 | **4** | 4 |
| width≥32 share | 94.8% | **99.7%** | 99.7% |

### Latency (ms) and stages (per-sweep ms, 100k)

| scale | mean pre → post | p50 pre → post |
|---|---|---|
| 10k | 0.92 → 0.90 (−2.2%) | 0.90 → 0.87 |
| 50k | 1.85 → 1.83 (−1.1%) | 1.80 → 1.75 |
| 100k | 2.71 → 2.70 (−0.4%) | 2.66 → 2.62 |

100k stages: posting_visit 64.051 → 62.955; scratch_flush 45.332 → 44.950;
scorer_batch 42.472 → **42.118 (−0.8%)**.

### Finding: the Task 135 +7.2% scorer delta was NOT flush-count-driven

Task 135 packet 002 measured dense scorer_batch at 42.8 vs row 39.9 ms
(+7.2%) and attributed it to the +470 boundary-drain flushes. This A/B
falsifies most of that attribution: with flush count and width histogram
restored EXACTLY to row levels, scorer_batch recovers only 0.35 ms of the
~2.9 ms gap. The residual dense-vs-row scorer difference must come from
elsewhere (candidate ordering/locality of dense-coalesced payload copies
feeding the kernel, or cross-cell noise between the separate row/dense
tables). Recorded as a source-grounded correction to the 135 packet's
follow-up framing.

### Storage

Same tables both cells; unchanged (`storage-dense-real*.log` per run dir).

## Not committed

- `baseline/truth-cache/`, `drainfix/truth-cache/` (gitignored).
