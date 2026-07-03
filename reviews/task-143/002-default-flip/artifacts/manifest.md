# Task 143 packet 002 — default-flip confirming cell: manifest

- Code + dylib + runner: `815518d82`; in-suite `precheck-build-sha.log`
  records the sha AND `current_setting('ec_ivf.turboquant_scorer')` =
  `int8_approx` (the flipped default, no session GUC set).
- Host/lane: m5-local PG18 (socket /Users/peter/.pgrx:28818), 2026-07-02.
- Fixture: fresh `task143flip_default_real100k` load from the staged 100k
  corpus with NO reloptions and NO session GUCs — the pure out-of-the-box
  path. nprobe sweep [8,16,24,32,40,48,64] recall (32q), latency [32,40]
  (32 iters, warm), storage.
- Command: `ecaz bench suite run --config
  reviews/task-143/002-default-flip/task143-default-flip-suite.json ...`
  (5/5 steps, exit 0).
- Key lines: recall 0.7844/0.8344/0.8750/0.8938/0.9031/0.9125/0.9250 —
  identical to packet 001's explicit dense-int8 cell at every point;
  latency mean 1.71 (n32) / 1.94 ms (n40); index 81.7 MiB (dense size;
  row builds measured 90.4 MiB).
- truth-cache/ not committed (gitignored).
