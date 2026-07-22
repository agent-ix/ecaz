---
task: 194
packet: 007-fixed-work-candidate
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 194 fixed-work wider/fewer-round candidate

The canonical release attribution selected one bounded traversal candidate:
compare production BW=4/H=100 against BW=8/H=50 on the same immutable
generation. Both arms retain the same 400-expansion cap, exact trained head,
head seeds, RaBitQ stored neighbor values, lazy10 rerank, and uncached Task
192/193 owner settings. Only the traversal beam/round shape changes.

Pre-registered prediction: wider expansion batches should reduce sequential hop
rounds and transport-wait remainder. Owner service per round may rise, and the
actual number of expanded nodes need not remain identical, so the decision is
based on recall, end-to-end mean/tails, and the nine-way work attribution—not
on the nominal cap alone. Recall loss or no useful end-to-end improvement is a
STOP. A useful isolated 100k result advances to the required 10k/50k/100k
matrix.

The suite runner and fixture now accept per-variant beam width and hop rounds,
so both arms are measured against one shared physical generation in one suite
step rather than separate, confounded builds.

Implementation: `e444f6474`.
Evidence metadata and the checked-in suite are in `artifacts/manifest.md`.
