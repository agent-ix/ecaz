# Task 184 materialization-plan manifest

- Source head: `eafcb6bae775aa6a6bc65ffc17d22d8023505e12`
- Task bucket / packet: `reviews/task-184/001-materialization-plan/`
- Lane: plan-only attribution and candidate-selection contract
- Baseline source: Task 183 packet 005 `run/results.jsonl`, SHA-256
  `e3fd4f51b47af43aee7406db90eef3de07d0689cb03fab887f6680628a0c0688`
- Baseline installed release head:
  `97cd5a76a5ea2d20ef94078566f66f85dacc97b2`
- Baseline: 100k, three exact/disjoint owners, trained production head,
  cap 4,096, 32 seeds, BW4/H100, RaBitQ, exact final ranking
- Baseline result: recall 0.9625; warm mean/p50/p95/p99/max
  40.20/39.20/51.50/56.30/57.90 ms; remote materialization
  26.955257 ms/query
- Environment inspection: canonical 10k/50k/100k staged symlinks present,
  PG18 reports 18.3, release/debug CLI binaries present, installed benchmark
  extension SHA-256
  `58a2af361807a98b8ec37dd9ad0f32b15bf4738539273915ea0513078550dfe2`
- Tests / benchmarks: not run; plan-only checkpoint
- Production/format effect: none

## Artifacts

- `materialization-attribution-contract.md`: frozen stage boundaries, nesting,
  work counters, fixture, result requirements, and candidate trigger.
