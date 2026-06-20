# Review Request: Task 111h Cross-Scale Matched-Recall V7 Decision Slice

This packet requests review for a derived cross-scale matched-recall analysis
over the committed post-v7 10k, 50k, 100k, and 1M Task 111h benchmark packets.

No new benchmark suite was run. The packet adds:

- `artifacts/select-matched-recall.jq`
- `artifacts/cross-scale-matched-recall-v7.md`
- `artifacts/manifest.md`

The selection rule matches packet 025: for each recall target and format, select
the lowest-p50 row that reaches the target; if no row reaches the target, report
the maximum-recall row as `NO_REACH`.

Targets covered:

- recall@10 >= `0.95`
- recall@10 >= `0.97`
- recall@10 >= `0.99`

Main readout:

- `source/f32` is the warm-cache local reference/default on the 50k, 100k, and
  1M matched-recall frontier.
- Current `index/f16` is recall-neutral but storage-heavy; it is not a
  promotion candidate in the current layout.
- `index/rabitq4` does not reach `0.95` at 50k or larger.
- `index/rabitq8` and `index/turboquant` can reach `0.95`, but not `0.97` or
  `0.99`, at 50k/100k/1M in the current grid.
- TurboQuant is the best compact index-side candidate in this warm-cache local
  slice, but still does not beat source/f32 at matched recall.

Please review whether the selected rows are correctly derived from the cited
JSONL artifacts and whether the warm-cache local decisions are appropriately
limited.

Non-claim: this is not a final Task 111h closeout. It does not supply
cold/remote evidence, table-owned compact storage evidence, legacy `0x2A`
baseline evidence, or new correctness fixtures.
