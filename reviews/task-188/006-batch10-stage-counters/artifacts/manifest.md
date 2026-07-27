# Task 188 batch-10 stage-counter follow-up manifest

- Task bucket: `reviews/task-188/006-batch10-stage-counters/`
- Reviewer source: `reviews/task-188/005-batch10-reconfirmation/feedback/2026-07-26-01-reviewer.md`
- Code checkpoint: `c810b6e5e`, parser branch plus focused test
- Preregistered config: `task188-batch10-stage-counters-suite.json`
- Config SHA-256: `ad3cd8064cad5b26c06c0e10a0e34052c0c6c8bf08086bbc97adee1f54b6ec35`
- Diagnostic result: failed during 100k build with `ENOSPC`; no result rows
  from that failed run are included in the decision evidence
- Durable failure note: `artifacts/stage-counter-diagnostic.md`
- 2026-07-27 rerun: physical setup passed, but the latency backend reached
  approximately 52 GB RSS and was terminated before emitting results;
  `artifacts/run/rerun-20260727/outcome.md`
- Accepted decision source: packet 005's re-normalized
  `artifacts/run/results.jsonl`, SHA-256
  `a1e0f57d9f18cfdd5d7ac1c6ed15dc70b947e655838093e939a770dc587c474e`

## 2026-07-27 efficient rerun

- Code checkpoint: `193cff682`
- Config: `artifacts/task188-batch10-stage-counters-efficient-suite.json`
- Config SHA-256: `d4c18956f06f139707cff700bbb4afdff7027050614dee3df78efe29445bb8af`
- Artifact root: `artifacts/run/efficient-20260727-r2/`
- Suite manifest SHA-256: `9c5e700417c6a1d8a90ea7c5bc314697e3e4e6bcb2b4806029a08c8a5207e74c`
- Results SHA-256: `5c430ffd988dd99b4a67dde675de31df4e2257f5e0616558c93846973c91898a`
- Summary SHA-256: `4a9ae48d4130155567e5eadbdf72bc22a20f72d55ffd37c9af8982e7405ec514`
- BW4 latency log SHA-256: `242ed8c31af8c9d11a5b3e3ee62de368028418c5f13e5a488bd0363fdfaebd0b`
- BW8 latency log SHA-256: `26593aa2641d05d1923b747420cf9f819f17ebb68d62ef5d4e38c67cdb23ba88`

The suite succeeded with the physical serving/topology gate, two 50-query
physical latency arms, 37 stage rows per arm, 28 materialization-work rows per
arm, and passing traversal reconciliation. It used `stage_counter_only=true`
and `benchmark_backend_batch_size=5`; therefore it intentionally has no recall
rows and its single-index storage fields are zero. Packet 005 remains the
recall/storage decision source. The direct batch-10 stage attribution and
resource observations are summarized in
`artifacts/run/efficient-20260727-r2/outcome.md` and
`artifacts/run/efficient-20260727-r2/resource-checks.md`.
