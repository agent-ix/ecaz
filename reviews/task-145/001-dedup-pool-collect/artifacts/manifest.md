# Task 145 packet 001 — dense-pool topk_collect A/B: artifact manifest

- Code under review: `0679ac536` ("Dedup IVF candidates into a dense pool;
  collect walks the pool, not the map", branch `task-145-topk-collect` off
  main `689528007`; code-identical parent = installed `815518d82`, the
  Task 143 default-flip commit — commits between are docs/packets only,
  verified via `git diff --stat 815518d82..689528007 -- src/ crates/` empty).
- Task bucket / packet: `reviews/task-145/001-dedup-pool-collect/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port
  28818, db `tqvector_bench`. 2026-07-03.
- A/B form: **before/after commit**, same session, same tables,
  back-to-back suite runs with one dylib swap between them:
  - baseline run: `artifacts/baseline/`, `precheck-build-sha.log` must
    record `ecaz_build_git_sha()` = `815518d82...` (pre-change).
  - pool run: `artifacts/pool/`, `precheck-build-sha.log` must record
    `0679ac536...` (dense-pool collect).
  - No installs mid-run; install evidence + dylib timestamps recorded
    below after each swap.
- Change under test: query-side only — the scan's candidate dedup
  structure (`CandidateDedupPool`: heap_tid -> u32 slot map + dense
  candidate vec) and the `topk_collect` walk source (contiguous pool
  slice instead of hash-map iteration). No on-disk, reloption, GUC, or
  build-path change; dedup semantics and explain counters unchanged;
  results byte-identical by construction (strict-total-order top-k is
  iteration-order independent; unit test `candidate_dedup_pool_collect_
  matches_reference_map_dedup` proves pool-vs-map equality).
- Fixtures (isolated one-index-per-table, dbpedia-openai3 1536-dim,
  seed 42, TurboQuant no-QJL 4-bit, pure current defaults: dense posting
  blocks auto + int8_approx scorer):
  - 10k: `task145_default_real10k` — fresh load this packet from
    `data/staged-current/ec_real_10k_*`.
  - 50k: `task145_default_real50k` — fresh load this packet from
    `data/staged-current/ec_real_50k_*`.
  - 100k: `task143flip_default_real100k` — reused Task 143 packet 002
    pure-default table (loaded 2026-07-03 at `815518d82`).
  - 1m tier (990k anchor split): `task143_dense_1m` — reused Task 143
    packet 001 table (`dense_posting_blocks=1` reloption, loaded
    2026-07-03 at `e6b08f497`; loader/build untouched since).
- Suite: `task145-dedup-pool-ab-suite.json` (this packet). Bespoke
  config reason (standard-sweep rule): before/after-binary A/B on fixed
  existing tables at the current defaults — the canonical lane config
  would reload every scale and bench all four AMs; this A/B needs the
  ec_ivf lane only, with the registered ec_ivf default recall grid
  [8,16,24,32,48,64] kept verbatim, latency at nprobe [32,40] for
  comparability with the Task 143 promotion baseline, queries_limit
  32/iterations 32 (24/16 at 1m, matching Task 143), stage counters on.
- Runner: single `target/release/ecaz` built at `0679ac536` (worktree
  `.task-worktrees/task-145`) for BOTH runs — runner-side code does not
  execute the changed scan path (it runs in the server dylib); one
  binary for all cells:

  ```sh
  ./target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-145/001-dedup-pool-collect/task145-dedup-pool-ab-suite.json \
    --artifact-dir reviews/task-145/001-dedup-pool-collect/artifacts/{baseline,pool}
  ```

- Shared ground truth: `truth-cache/` at the packet root (gitignored,
  regenerable) is shared by both runs, so recall in both cells is scored
  against the identical truth set.
- Microbench artifacts (pre-A/B attribution, `#[ignore]`d unit test
  `candidate_dedup_profile_map_walk_vs_pool_walk`, release, 45k
  candidates / width 50 / 200 iters):
  - `artifacts/profile-map-vs-pool-collect.log` — map walk = 60.3% of the
    pre-change collect; pool walk = 10.2% of the post-change collect;
    pool collect = 65.5% of map collect hot-in-cache.
  - `artifacts/unit-dedup-pool.log` — the two dedup-pool unit tests.
- Validation: `cargo clippy --all-targets --no-default-features
  --features pg18` clean on touched files; `cargo test --release --lib`
  for the scan tests. pgrx runtime tests deferred per the known macOS
  `_BufferBlocks` dyld blocker (compile gates + e2e suite instead).

## Cells

Three cells, one commit per axis (A/B per change):

1. `artifacts/baseline/` — dylib `815518d82` (pre-change code), in-suite
   precheck sha confirmed; post-run sha re-checked unchanged.
2. `artifacts/pool/` — dylib `115d60816` = code commit `0679ac536`
   (lever 1: dense dedup pool; `115d60816` is docs-only on top).
   Install log + shasum: `artifacts/install-pool-dylib.log`.
3. `artifacts/lazyheap/` — dylib `22411e3dd` (lever 2: lazy min-heap for
   the unbounded collect, on top of lever 1).
   Install log + shasum: `artifacts/install-lazyheap-dylib.log`.

All three runs: 17/17 steps succeeded (`suite-manifest.json` per cell).

## Key result lines

### Recall — byte-identical (the gate)

All 24 recall cells (4 scales × 6 nprobe points) are equal across all
three binaries, digit for digit (`results.jsonl` per cell;
`recall@k` values e.g. 100k n32 0.8938, 1m n32 0.9208 — identical to
the Task 143 packet 002 defaults). Ground truth shared via the packet
`truth-cache/`. Zero mismatches.

### Latency (mean ms, warm, k=10; n32 / n40)

| scale | baseline | pool (lever 1) | lazyheap (levers 1+2) | lazyheap vs baseline |
|---|---|---|---|---|
| 10k  | 0.63 / 0.72 | 0.61 / 0.68 | **0.58 / 0.64** | **−7.9% / −11.1%** |
| 50k  | 1.14 / 1.34 | 1.19 / 1.34 | 1.15 / 1.34 | +0.9% / +0.0% |
| 100k | 1.81 / 2.12 | 1.71 / 1.96 | **1.63 / 1.83** | **−9.9% / −13.7%** |
| 1m   | 7.37 / 8.50 | 7.45 / 8.53 | **6.76 / 7.81** | **−8.3% / −8.1%** |

### topk_collect stage (per-sweep ms, nprobe 32; scans: 32 at ≤100k, 16 at 1m)

| scale | baseline | pool | lazyheap | lazyheap vs baseline |
|---|---|---|---|---|
| 10k  | 1.836 | 1.769 | 0.416 | −77% |
| 50k  | 4.483 | 4.446 | 1.253 | −72% |
| 100k | 7.042 | 6.561 | 1.895 | −73% |
| 1m   | 17.126 | 16.444 | **4.166** | **−76%** |

Attribution per lever: lever 1 (pool walk) moved the stage only
−4..−7% — the pre-change stage cost is dominated by the full
O(n log n) sort in the UNBOUNDED collect path (no pre-rerank limit at
the shipping defaults: rerank mode is not heap_f32, `exact_rerank`
samples = 0 in every cell), which the microbench's bounded shape did
not model. Lever 2 (lazy heap, sort removed) is the real win. 1m
per-query: ~45k deduped candidates (task87 counters: 721,524
candidates / 16 scans), topk_collect 1.07 ms → 0.26 ms.

Under the lazyheap binary the 1m/n32 approximate-scan budget is:
posting_visit 84.5 (page access 66.0 + parse/push) / scorer_batch 31.9
/ scratch_flush 41.6 / candidate_record 8.1 / probe_plan 7.0 /
topk_collect 4.2 ms per sweep — topk_collect drops from ~17% of the
scan to ~4.5%.

The 50k e2e flatness is session noise, not a missing stage win (stage
−72% there too; 50k also wobbled +4.4% between the two IDENTICAL-code
baseline/pool binaries at n32, and its stage saving ~0.1 ms/query is
within that wobble).

### Storage

100k and 1m: every storage field identical across all three cells (the
change is query-side only). The fresh 10k/50k tables show +1.6 B/row
(10k) and +0.1 MiB total (50k) heap-side drift between the baseline and
pool runs — autovacuum/FSM settling on tables loaded minutes earlier;
identical between pool and lazyheap; no index-size change anywhere.

## Run log

- 2026-07-03 07:59 local: baseline suite (dylib `815518d82`, in-suite
  precheck + post-run sha check).
- 2026-07-03 08:05: lever-1 dylib installed (shasum verified vs
  `target/release`), pool suite run (precheck `115d60816`).
- 2026-07-03 08:23: lever-2 dylib installed (shasum verified), lazyheap
  suite run (precheck `22411e3dd`); post-run sha unchanged.
- No mid-run installs in any cell (sha checked before and after each).
