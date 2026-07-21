# Task 187 post-materialization baseline manifest

- Task bucket: `reviews/task-187/`
- Packet: `reviews/task-187/001-post-materialization-baseline/`
- Baseline head / runner SHA: `fe98ea2cc2273465ae28a942b7dacf9ca347a0de`
- Lane: Intel local, PG18, three loopback PostgreSQL owners
- Retained policy: `training_landmarks_exact`, cap 4,096, exact landmark
  scoring, 32 returned seeds, BW4/H100, graph degree 32, RaBitQ neighbors,
  production lazy10 payload windows
- Evaluation: staged `ec_real_100k`, 200 held-out queries, top-k 10
- Latency: 50 timed samples after 10 warmups, concurrency 1, warm cache
- Isolation: fresh one-index-per-table physical generation through the checked-
  in suite config; no shared benchmark fixture
- Runner: `ecaz bench suite`

- Config SHA256: `c19086675a718c071786cf286dae6b5ef645ad4b5dabb8103e1989faa1b06a7e`
- Command: `/home/peter/dev/ecaz/target/release/ecaz bench suite run --config reviews/task-187/001-post-materialization-baseline/artifacts/post-materialization-baseline-suite.json`
- Status: succeeded; audit passed; one step completed, zero failures/skips.
- Result artifacts: `run/results.jsonl` SHA256
  `2f1bd8dd38090057751fd3905a5bae257aedd964193af466cc06384036a44168`;
  `run/suite-manifest.json` SHA256
  `1a6cff49f2f4c3c6ea2416e0de5068a87d1c9b422ccc0572bb2785d59f5f75f5`;
  compact summary SHA256
  `134fc929f5c409f932841243e6f6ec7e82317b9c15ffe048252afca2cd1af332`.
- Key results: recall `0.9625` (CI95 `0.9532–0.9700`); warm mean/p50/p95/p99/max
  `22.40/22.20/25.60/26.80/27.30 ms`; storage generation
  `2,496,626,688` bytes, control `24,576`, coordinator source
  `1,666,260,992`, single index `854,810,624`.
- Traversal attribution (stage counters): total `7.468375 ms`; local expand
  `1.229638 ms`; remote expand `6.174150 ms`; derived control/merge remainder
  `0.064587 ms`; head score `2.145140 ms`; seed select `0.094457 ms`.
- Materialization is separately accounted for: remote materialize
  `10.018400 ms` (production lazy10), so it is not folded into traversal.
- Provenance: `ec_real_100k`, query SHA
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`, head
  digest `50261d7627471fa3329535cd017ead6102cb220c62ca12dc9715178d05333b54`,
  3 owners, isolated one-index-per-table physical fixture.
