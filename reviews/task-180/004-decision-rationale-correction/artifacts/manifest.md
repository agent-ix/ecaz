# Task 180 decision-rationale correction manifest

- Head at correction start: `71690b6c4c06e4457c97a3254c6b6b53bea916d4`
- Task bucket / packet: `reviews/task-180/004-decision-rationale-correction/`
- Lane: documentation-only correction; no benchmark rerun
- Source evidence: `reviews/task-180/003-full-scale-decision/`
- Command: none
- Timestamp: 2026-07-15 America/Los_Angeles

The unchanged 100k result is production 0.9275 recall / 40.3 ms p50 versus
width64/seeds64 0.9280 / 40.9 ms. The recall confidence intervals overlap and
the candidate is slower, supporting a relative A/B NO-GO for that tuning
direction independently of the unapproved proposed NFR-017 numerical targets.
