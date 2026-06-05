# Task 82: SPIRE 1M Recall Attribution

Status: complete (2026-06-05)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (Task 81 / AWS 1M follow-up)

## Why

Tasks 79 and 81 established a credible local/AWS 100k latency path at the
Task 79 acceptance point, but the retained AWS 1M/q500 surface still misses the
high-recall target:

- Task 81 1M q500 retained shape: recall@10 `0.9832`, candidate_sum
  `9,213,846`.
- Current HEAD rerun packet
  `benchmarks/aws-spire-1m-task81-head-rerun/001-run/`: recall@10 remains
  `0.9832` with the same candidate surface, while p50 improves to
  `250.251 ms`.
- The wider top-graph recall-ceiling packet
  `benchmarks/aws-spire-1m-topgraph-rebuild/001-run/` reaches recall
  `0.9976-1.0000`, but only by expanding q500 candidates to
  `251,510,240-495,000,000` and p50 to `554-1039 ms`.

The next SPIRE question is therefore not whether we can make the failing
candidate surface a little faster. It is why the missing q500 truth neighbors
are absent, and which narrow mechanism can recover recall without returning to
the full wide-top-graph candidate explosion.

## Scope

Build a packet-backed attribution workflow for the retained AWS 1M/q500 SPIRE
surface. The workflow should identify, per missed truth neighbor, whether the
miss happened because:

- routing did not select the leaf containing the truth row;
- leaf/block pruning selected the leaf but skipped the row's block;
- candidate scoring saw the row but candidate or rerank caps truncated it;
- heap rerank or result merge changed the final top-k ordering.

The implementation may add narrow CLI or SQL diagnostic surfaces if the current
`ecaz bench suite` and `bench spire-pipeline` outputs cannot answer the
question. Do not add one-off shell sweepers; extend `ecaz bench suite` or the
existing SPIRE pipeline diagnostics instead.

## Required Evidence

- Use `ecaz bench suite` for all measurement runs.
- Reuse the retained AWS 1M/q500 truth cache from
  `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`.
- Preserve AWS cost hygiene: pause `1m` after each run and capture packet-local
  final status.
- Capture all durable benchmark evidence under `reviews/task-82/` or an
  immutable `benchmarks/` packet cited by `reviews/task-82/`.
- Compare against Task 79/81 optimized baselines, not the old 15M full-leaf
  row from pre-Task-79 work.

## Gates

- Produce an attribution table over the q500 missed truth rows showing counts
  by miss stage.
- Identify the dominant miss stage with enough evidence to choose the next
  implementation slice.
- Any proposed recall-recovery slice must target recall above `0.9832` without
  exceeding the old q500 candidate surface by more than a justified bounded
  amount.
- Do not accept a solution that only reproduces the `tg256` full-candidate
  recall ceiling unless the task explicitly closes as "not yet competitive."

## Exit Criteria

- A review packet under `reviews/task-82/` records the attribution method,
  command(s), suite config(s), artifacts, and key result rows.
- If a narrow implementation slice lands, it has a matching packet with PG18
  validation and AWS 1M/q500 evidence.
- If no narrow slice is justified, close the task with a concrete follow-up
  recommendation: routing breadth, block scoring/pruning, candidate cap policy,
  or default/preset deferral.
- AWS `1m` is paused at closeout.

## Closeout

Closed by `reviews/task-82/001-aws-1m-recall-attribution/`.

The bounded AWS 1M/q500 attribution run preserved the Task 79/81 retained shape
(`nprobe=96`, `rerank_width=25`, global block budget `1152`) and reproduced
`recall@10=0.9832` with `candidate_sum=9,213,846`. Across the `5,000` q500
truth rows, `4,916` were hits and `84` were misses:

- `routing_miss`: `3`
- `selected_leaf_block_pruning_or_candidate_cap`: `81`
- `assignment_missing`: `0`
- `candidate_or_rerank_cap`: `0`

The dominant miss stage is therefore not top-graph routing breadth. The next
SPIRE recall-recovery slice should target selected-leaf block containment and
block scoring/pruning recovery, ideally starting with a target-only
selected-block containment diagnostic so the remaining `81` misses can be split
between exact block-pruning loss and candidate-budget truncation without the
slow full block-rank helper. The AWS `1m` profile was paused at closeout.
