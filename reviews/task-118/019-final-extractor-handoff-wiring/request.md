---
task: 118
packet: reviews/task-118/019-final-extractor-handoff-wiring
checkpoint_sha: 2fb288b30f3aa131da68848a1de1110bd962b01d
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Final Extractor Handoff Wiring

## Scope

This checkpoint wires packet 018's final-table extractor into the main Intel
closeout runbook and audit template.

Updated:

- `reviews/task-118/010-intel-closeout-runbook/artifacts/intel-closeout-runbook.md`
- `reviews/task-118/011-final-closeout-audit-template/artifacts/final-closeout-audit-template.md`

The runbook now writes:

`reviews/task-118/006-final-attribution-matrix/artifacts/final-decision-table-intel.tsv`

from the final Intel `results-10k-intel.jsonl`, `results-50k-intel.jsonl`, and
`results-100k-intel.jsonl` files.

## Validation

No benchmark was run. Packet 018 already validates the extractor against the
available 10k rows. This packet is a handoff wiring update so the final Intel
operator uses that extractor instead of manually joining recall, frontier,
score-correlation, and storage result rows.

## Remaining Task 118 Closeout Work

Run the Intel 10k/50k/100k suites, generate
`final-decision-table-intel.tsv`, fill the dominant-loss and next-action
columns, and update packet 006.
