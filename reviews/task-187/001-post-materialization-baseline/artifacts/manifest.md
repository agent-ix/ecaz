# Task 187 post-materialization baseline manifest

- Task bucket: `reviews/task-187/`
- Packet: `reviews/task-187/001-post-materialization-baseline/`
- Baseline head: `origin/main` after Task 191 merge (record exact SHA before
  running the suite)
- Lane: Intel local, PG18, three loopback PostgreSQL owners
- Retained policy: `training_landmarks_exact`, cap 4,096, exact landmark
  scoring, 32 returned seeds, BW4/H100, graph degree 32, RaBitQ neighbors,
  production lazy10 payload windows
- Evaluation: staged `ec_real_100k`, 200 held-out queries, top-k 10
- Latency: 50 timed samples after 10 warmups, concurrency 1, warm cache
- Isolation: fresh one-index-per-table physical generation through the checked-
  in suite config; no shared benchmark fixture
- Runner: `ecaz bench suite`

The final manifest will record the exact head SHA, suite SHA, corpus/query
digests, command output, and key attribution lines after the run.
