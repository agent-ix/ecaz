# Task 111b Completion Audit

Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`

Conclusion: Task 111b's scoped acceptance criteria are complete. The format is **not** promoted as the durable/default layout; that decision is explicitly deferred to Task 111c/111d after score-in-place and storage-density work.

## Acceptance Criteria

### AC1: Columnar frozen-list format implemented behind a gate; deterministic build.

Status: complete.

Evidence:

- `reviews/task-111b/001-columnar-header-format/` reserved and validated tag `0x29` v1.
- `reviews/task-111b/002-columnar-buffer-chunks/` added the parallel LE column model and page-aware whole-item chunking.
- `reviews/task-111b/003-columnar-build-writer/` added `columnar_frozen_lists = 1` build-time writer.
- `reviews/task-111b/005-columnar-placement-validation/` closed the raw-page placement read-back risk.
- Reviewer feedback for packets 001, 002, 005 is LGTM; packet 003's required follow-up was closed by packet 005.

### AC2: Existing row / dense (`0x25`) / aligned (`0x28`) indexes remain readable.

Status: complete.

Evidence:

- `reviews/task-111b/006-format-compatibility-tags/` added/read-backed old-format coverage and documented the tag set.
- Reviewer feedback for packet 006 says it closes AC2 and AC6.
- `docs/on-disk-format.md` records row `0x23`, dense `0x25`, packed experimental `0x26`/`0x27`, aligned dense `0x28`, and columnar `0x29`.

### AC3: Recall and NDCG unchanged vs legacy path for all compared cells.

Status: complete for the 111b benchmark matrix.

Evidence:

- `reviews/task-111b/008-columnar-benchmark-matrix/artifacts/suite-status.log`: `completed=50 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `reviews/task-111b/008-columnar-benchmark-matrix/artifacts/summary.md` reports recall and NDCG for 50k/100k x TQ/rb1/rb2/rb4/rb8 and states recall matches the 111a storage surfaces for each scale/quant/nprobe cell.
- Reviewer feedback for packet 008 verified committed `results.jsonl`/`summary.md` and says recall parity holds.

### AC4: Mixed frozen-column + delta-row scan and vacuum return same candidates as legacy under controlled fixtures, including after deletes.

Status: complete.

Evidence:

- `reviews/task-111b/004-columnar-scan-vacuum/` adds the copy-based columnar scan and vacuum fixture.
- Reviewer feedback for packet 004 verifies scan skips deleted postings, vacuum marks the per-list bitmap size-preservingly, row delta tuples in the same list continue to scan, and the PG18 fixture covers build scan, mixed insert scan, vacuum deletion, directory counts, and post-vacuum rescan.
- `reviews/task-111b/007-columnar-scan-counters/` adds dedicated counters and verifies columnar postings no longer charge dense counters.

### AC5: Benchmark packet reports storage and posting-pages-read vs Approach A and 111a dense formats across TQ + RaBitQ {1,2,4,8} at 50k/100k, plus recall/latency.

Status: complete as a baseline packet.

Evidence:

- `reviews/task-111b/008-columnar-benchmark-matrix/` is the benchmark packet.
- `artifacts/manifest.md` records 10 isolated cells: 50k/100k x TQ/rb1/rb2/rb4/rb8.
- `artifacts/summary.md` reports recall, NDCG, latency p50/p95/p99, index size, bytes/row, EXPLAIN posting pages, logical bytes copied, and storage comparison against 111a row/dense-old/dense-a/dense-b.
- Reviewer feedback for packet 008 says 111b AC5 is met as a baseline.

### AC6: Packet records on-disk tag/version set and Task 42 reconciliation plan.

Status: complete.

Evidence:

- `reviews/task-111b/006-format-compatibility-tags/` records the tag set and compatibility status.
- `docs/on-disk-format.md` records `0x29` v1 and states `0x26`/`0x27` are abandoned page-spanning experiments, not promotion candidates.
- Reviewer feedback for packet 006 says it closes AC6 / Task 42 recordkeeping.

## Non-Goals And Deferrals

The following are not 111b completion requirements and remain intentionally deferred:

- zero-copy page-aware scatter scorer: Task 111c;
- pre-transposed canonical geometry: Task 111d;
- default promotion decision: deferred until 111c/111d evidence;
- storage-density improvements identified by packet 008 feedback: carry into 111c/111d work.

## Closeout Decision

Task 111b is complete because its durable format, correctness, compatibility, vacuum/mixed-scan, counter, and benchmark-baseline requirements are implemented, reviewed, and documented. The status update marks the task complete while preserving the no-promotion decision and the follow-up risks surfaced by packet 008.
