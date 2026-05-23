# Task 50/449: HNSW bench window — §Exit Criteria #3 closed

Closes the last outstanding §Exit Criteria gate flagged by
`reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`
§"Bench gate" and explicitly recommended as next action by
`448/feedback/2026-05-23-01-reviewer.md`:

> Recommend the agent's next action: **run the bench window**, not
> open a new code slice.

Evidence is a pure-measurement packet at
`benchmarks/task-50-m5-hnsw-baseline/` (canonical placement per
CLAUDE.md §"Benchmark Data Packets"), driven entirely by
`ecaz bench suite` per the §"Benchmark Runner" mandate. No bash glue.

## Scope honored

- HNSW only, per the post-burndown scope-lock recorded in
  `448/feedback/2026-05-22-02-reviewer.md`. No IVF / SPIRE / DiskANN
  / AM-wide P3 / AM-wide P6 work in this packet.
- `bench suite` config covers `load` + `recall` + `latency` + `storage`
  at `ec_real_10k` and `ec_real_100k` on the local M5 Pro fixtures.

## Head + host

| Field | Value |
| --- | --- |
| HEAD SHA | `18acf379a` (post-merge `ebb022a7a`, IVF build-fix `54a2c1409`) |
| Host | Peters-MBP — Apple M5 Pro, 64 GiB, macOS 26.4.1 (arm64) |
| PostgreSQL | 18.x pgrx local install at `/Users/peter/.pgrx:28818` |
| Extension build | `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` (replaced stale May-18 dylib) |

The pre-burndown WSL2 baseline at `benchmarks/task-50-local-baseline/`
is **not directly comparable** (Intel WSL2 vs Apple M5 Pro). These
numbers establish the new M5 reference; future Task 50 / Task 33
packets compare against them on the same host.

## Headline results

Full tables: `benchmarks/task-50-m5-hnsw-baseline/manifest.md`
§"Key results". Per-step logs + `suite-manifest.json` +
`results.jsonl` under that packet's `artifacts/`.

Recall@10 (m=16, k=10, ip metric):

| ef_search | 10k recall | 100k recall | 10k mean | 100k mean |
| ---: | ---: | ---: | ---: | ---: |
| 40  | 0.9040 | 0.7426 | 0.65 ms | 1.05 ms |
| 80  | 0.9530 | 0.8506 | 0.90 ms | 1.65 ms |
| 120 | 0.9605 | 0.8973 | 0.92 ms | 2.10 ms |
| 200 | 0.9775 | 0.9414 | 1.11 ms | 2.83 ms |
| 400 | 0.9950 | 0.9676 | 1.72 ms | 4.79 ms |

Index sizes (m=16, ef_construction=128): 13.0 MiB / 10k, 130.2 MiB / 100k
(~1365 B/row — flat per-row cost confirms the metadata page is amortized
by ≥10k).

## §Exit Criteria status

Refresh of the 448 closeout table:

| Criterion | Status |
| --- | --- |
| Densest residual modules processed at least once | ✓ All 10 HNSW files (448 closeout) |
| Each processed module dropped ≥30% **OR** structural ceiling documented | ✓ -40.44% headline; 3 sub-30% files documented (448 closeout) |
| **No bench lane regresses beyond tolerance** | **✓ M5 Pro HNSW baseline captured this packet — recall@10 monotonic in ef_search, latency in line with M5 Pro ARM perf expectations, no failure modes observed** |
| Closing summary packet records final per-module distribution + names next-highest-density modules | ✓ (448 closeout) |

**All four §Exit Criteria satisfied.** Task 50 HNSW lane closes.

## Tolerance note

The §Performance Gate references "tolerance same as Tasks 31-33." No
prior M5 Pro reference existed for HNSW (Task 33 §Phase 1 explicitly
called for an M5 reference refresh as a future step, and Task 26
remained the standing reference on different hardware). This packet
serves *as* that M5 reference. There is no within-host before/after
A/B; the structural argument in `448` §"Bench gate" (no
allocation-shape / scoring-math / WAL-ordering / payload-byte changes
across the 47-packet rotation) stands as the substantive
neutrality claim, and these numbers are the empirical headline that
future packets can regress against.

## macOS dyld unblock (separately committed)

This run required first fixing a macOS-26 dyld blocker that prevented
the post-burndown `ecaz-cli` binary from starting at all. The fix is
17 PG `static mut` stubs in `crates/ecaz-cli/src/pg_macos_stubs.rs`
(cfg target_os=macos), addressing the root cause described in the
updated `feedback_dyld_buffer_blocks_known` memory: the CLI
transitively links pgrx PG-backend symbols via `bench_api`'s
re-export graph, and macOS-26 chained fixups bind those eagerly.

That fix lands as its own commit (separately reviewable) and is the
gating change that lets `ecaz bench suite` run on the local M5 at all.

## Parallel-reviewer concur (already landed)

Two reviewer feedback files landed concurrently at the benchmark
packet during this session:

- `benchmarks/task-50-m5-hnsw-baseline/feedback/2026-05-23-01-reviewer.md`
  (commit `5a35f644c`) — planning-stage review of the packet
  scaffold; confirmed scope-lock honored, suite shape correct,
  `--bits 4` HNSW default a no-op (not a bug), and recommended
  the "functional + forward-baseline" gate interpretation rather
  than strict A/B (since the WSL2 baseline is on a different host).
- `benchmarks/task-50-m5-hnsw-baseline/feedback/2026-05-23-02-reviewer.md`
  (commit `1ffb2e32f`) — post-run review: 8/8 steps succeeded,
  recall monotonic in ef_search, p99/p50 ratio under 1.65× at all
  sweep points, storage per row stable. **Approve. §Exit Criterion
  3 met. Task 50 HNSW rotation closes.**

This request.md is the matching coder-side packet for the bench run
that those feedbacks reviewed. The bench evidence + my dyld unblock
+ the reviewer concur together close the gate.

## Artifacts cited

All under `benchmarks/task-50-m5-hnsw-baseline/artifacts/`:

- `suite-manifest.json` — 8/8 step status, full expanded commands
- `results.jsonl` — structured normalized rows for every measurement
- `suite-run.log` — combined suite stdout/stderr
- `corpus-load-ec_real_{10k,100k}-hnsw.log` — load + index build
- `recall-ec_real_{10k,100k}-hnsw.log` — recall@10 sweep tables
- `latency-ec_real_{10k,100k}-hnsw.log` — latency distribution tables
- `storage-ec_real_{10k,100k}-hnsw.log` — index + heap size tables
- `pgrx-install.log` — extension rebuild log (HEAD `18acf379a`)

## Validation skipped per CLAUDE.md

Per CLAUDE.md §"Coder Workflow / Checkpoint Rules", pgrx runtime
tests are skipped on macOS — the dyld blocker for pgrx-test remains
deferred to Linux per memory `feedback_dyld_buffer_blocks_known`.
The CLI-side dyld fix (this packet) is a separate problem from the
pgrx-test one and is validated by the bench suite actually running
end-to-end (8/8 succeeded). `cargo check` clean from the 448 closeout
still applies; no extension code changed in this packet.
