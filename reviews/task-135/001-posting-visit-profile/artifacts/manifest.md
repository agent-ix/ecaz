# Task 135 packet 001 — posting-visit sub-stage profile: artifact manifest

- Head SHA (timer code + dylib + runner): `8f7bce3cc`
  ("Add posting_page_decode stage timer...", branch `task-136-rank1-scorer`;
  the branch also carries the Task 136 GUC-gated scorer at `9514c7518`, which
  defaults OFF (`lut`) and is inert in this run).
- Task bucket / packet: `reviews/task-135/001-posting-visit-profile/`
- Host: Apple M5 Pro (m5-local), PG18 socket `/Users/peter/.pgrx` port 28818,
  db `tqvector_bench`. 2026-07-02.
- Installed dylib verified in-suite: `precheck-build-sha.log` records
  `ecaz_build_git_sha()` = `8f7bce3cc7bed148b97939d519665dcf922f1878` (= head).
  Release build (15.2 MB dylib installed 08:49 local; a stray DEBUG dylib from
  a `cargo test` pgrx run was detected and replaced before this suite —
  hazard noted for future runs).
- Fixture: latency-only profile on the **existing row-layout tables from the
  Task 136 packet run** (`task136_tq_ivf_real{10k,50k,100k}` prefixes, plain
  turboquant, dbpedia 1536-dim, loaded same-day at `9514c7518`; loader/build
  code unchanged by the timer commit). No loads in this suite — plain dylib
  swap, no extension recreate. nprobe [32], k 10, warm cache, concurrency 1,
  LUT scorer (default).
- Runner command:

  ```sh
  target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
    bench suite run --config reviews/task-135/001-posting-visit-profile/task135-posting-visit-profile-suite.json \
    --artifact-dir reviews/task-135/001-posting-visit-profile/artifacts
  ```

- Bespoke config reason: latency-only, no-load profile (stage attribution for
  a timer that cannot change scan results); the canonical lane matrix is not
  the object here.
- Completed 2026-07-02, exit 0, 5/5 steps (`suite-manifest.json`,
  `results.jsonl`, `suite-run.log`).

## Key result lines (per-sweep elapsed_ms from `[ivf-stage-counters]`)

From `latency-profile-real{10k,50k,100k}.log`:

| scale | posting_visit | posting_page_decode | scratch_flush | scorer_batch | candidate_record |
|---|---|---|---|---|---|
| 10k (64 it) | 50.580 | 35.911 | 28.126 | 26.230 | 1.745 |
| 50k (48 it) | 70.645 | 61.206 | 46.340 | 43.146 | 2.965 |
| 100k (32 it) | 86.012 | 63.888 | 45.198 | 42.165 | 2.747 |

Derived split of the Task 133 "page I/O + decode" share
(posting_visit − scratch_flush):

| scale | visit − flush | page/buffer access (visit − page_decode) | parse + scratch push (page_decode − flush) |
|---|---|---|---|
| 10k | 22.45 | 14.67 | 7.79 |
| 50k | 24.31 | 9.44 | 14.87 |
| 100k | 40.81 | 22.12 | 18.69 |

(The page-access column slightly overstates by the post-loop drain-flush time,
which runs inside posting_visit but outside the page callbacks; drain is ≤2
flushes per scan out of ~41.)

Kernel counters (100k): 331,757 candidates / 1,311 flushes per sweep
(~10.4k postings/query, avg flush width ~253; width_ge32 = 1307/1311) — flush
widths are healthy, so narrow-flush thrash is NOT the row-path problem.

## Interpretation (cited by request.md)

At 100k the non-flush posting-visit share splits ~54% page/buffer access
(22.1 ms/sweep = 0.69 ms/query) vs ~46% entry parse + scratch push (18.7 ms).
Per-posting parse+push is already ~56 ns (768 B payload memcpy + field
pushes) — near the copy floor. Page access at ~0.69 µs/page over ~1k
row-layout pages/query is near the PG pin/lock floor. Neither half has big
in-place headroom: the lever is page/entry COUNT, i.e. the posting layout —
row postings pack ~10/page while the shipped (gated) dense-block layout packs
hundreds and decodes SoA in one parse.
