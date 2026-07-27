# Task 186 capacity-config provenance manifest

- Task bucket: `reviews/task-186/006-capacity-config-provenance/`
- Review finding: `reviews/task-186/001-capacity-control/feedback/2026-07-26-02-reviewer.md`
- Original two-arm config:
  `reviews/task-186/001-capacity-control/artifacts/task186-capacity-control-100k-suite.pre16384.json`
- Original config SHA-256: `87e59b638f1572abd609d04c03aa8600014d63ee624de2a531455f0487bbdad6`
- Original run directory: `reviews/task-186/001-capacity-control/artifacts/run-benchmark-feature/`
- Amended conditional config:
  `reviews/task-186/001-capacity-control/artifacts/task186-capacity-control-100k-suite.json`
- Amended config SHA-256: `edbf80d33be171e7845507dd97450fad610d148d649eaf063bbcc797ce9deba8`
- Amended run directory: `reviews/task-186/001-capacity-control/artifacts/run-cap16384/`
- Validation: both files parse as `SuiteConfig`; the original contains exactly
  the 4,096 and 8,192 steps, while the amended config contains the conditional
  16,384 step as well.
