# Task 183 latency-attribution manifest

- Pre-registration head: `8c47c940830eab0c63f0e08cf886e80207dd2afb`
- Task bucket / packet: `reviews/task-183/005-latency-attribution/`
- Lane: benchmark-only stage-counter implementation and 100k profile; pending
- Retained policy: Task 182 training landmarks, cap 4,096, exact head scoring,
  32 seeds, BW4/H100, RaBitQ traversal, exact final ranking
- Evaluation input: held-out rows 1--200
- Initial profile: 100k, 50 timed latency queries after 10 warmups,
  concurrency 1, warm cache
- Isolation: fresh one-index-per-table physical generation through a future
  checked-in `ecaz bench suite` config
- Timestamp: 2026-07-17 America/Los_Angeles

No profile result or optimization decision is claimed yet.
