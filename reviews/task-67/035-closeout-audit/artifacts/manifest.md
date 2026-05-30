# Task 67 Packet 035 Artifact Manifest

- head SHA: a361d7d8553ca74492c16300be14bf19b2a04fa0
- task bucket: `reviews/task-67/035-closeout-audit/`
- timestamp: 2026-05-30T15:41:03Z
- lane: closeout audit after measured-scope amendment
- fixture / storage format / rerank mode: summarized from cited packets
- isolated one-index-per-table or shared-table surfaces: summarized from cited packets

## Artifacts

### `artifacts/preflight/cloud-status-closeout.log`

- command: `script -q -c "target/debug/ecaz cloud status --profile 10k-intel" reviews/task-67/035-closeout-audit/artifacts/preflight/cloud-status-closeout.log`
- result: passed
- key lines: `state: paused`; `cost: ~$0.00/hr running`

### `artifacts/local/git-status-closeout.log`

- command: `script -q -c "git status --short --branch" reviews/task-67/035-closeout-audit/artifacts/local/git-status-closeout.log`
- result: passed
- key lines: branch was in sync with `origin/main`; untracked files shown are pre-existing Task 30 / stale Task 67 artifact directories plus this packet before commit.

## Cited Evidence

- Task amendment: `reviews/task-67/033-measured-closeout-amendment/`
- RaBitQ quant validation and scan validation support: `reviews/task-67/034-ivf-adaptive-test-fixture/`
- bf16 decision: `reviews/task-67/029-bf16-decision/`
- bits=8 SQL headline measurement: `reviews/task-67/027-rabitq8-headline-sql-measurement/`
- bits=1 SQL/top-k frontier measurements: `reviews/task-67/022-topk-frontier-bits1-measurement/`, `reviews/task-67/026-dedup-reserve-bits1-measurement/`
- kernel benchmark evidence: `reviews/task-67/020-rabitq-kernel-bench/`, `reviews/task-67/023-rabitq8ls-kernel-bench/`
- differential scaffold and safety packets: `reviews/task-67/011-x86-differential-scaffold/`, `reviews/task-67/012-x86-scaffold-safety/`
