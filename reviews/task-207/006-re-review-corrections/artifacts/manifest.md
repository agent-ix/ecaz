# Task 207 re-review correction manifest

- code checkpoint: `366a7973d`
- task bucket: `reviews/task-207/006-re-review-corrections/`
- code correction: physical `head_construction` marker persisted and exposed
  by `ec_distann_active_head_construction()`
- evidence correction: release seed controls compiled out; effective seed
  count is `(beam_width * 2).max(32)` and must be reported as 256 for BW128
- owner lane disposition: withdrawn from membership/overlap decision evidence
  because it is head-independent and captured top-k 32
- search path disposition: persisted-head/Vamana remains diagnostic; no
  production default or `training_landmarks_exact` promotion is made here
- regression coverage: PG18 lifecycle assertion checks the persisted marker
  surface and `marker_attested=true`
- timestamp: 2026-08-04, America/Los_Angeles
