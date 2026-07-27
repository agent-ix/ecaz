---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
seq: 1
---

# Task 188 isolated BW8 candidate

Phase 1 selected only BW8/H100 for confirmation. It held the exact bounded
head seeds constant and improved 100k recall from 0.9740 to 0.9805 with nearly
flat mean latency, while BW2 and H50 were recall-neutral or worse. This packet
pre-registers the required A/B confirmation at 10k, 50k, and 100k, measuring
recall, warm latency, storage, build, head bytes, topology, and engagement for
BW4 control versus BW8 candidate.

The suite keeps graph degree, head policy/cap, neighbor code format, seed
width/count, topology, and query fixture fixed within each scale. It does not
combine a graph rebuild with an adaptive policy and does not alter production
defaults.

The full-scale suite completed successfully. BW8 remains the selected
research candidate: it is recall-neutral at 10k, gains 0.0025 recall at 50k,
and gains 0.0065 recall at 100k, with identical storage at every scale. The
50k latency cost is material, so this packet selects BW8 for a follow-up
productionization/acceptance task without changing a production default.

See `artifacts/task188-bw8-full-scale-results.log` for the cited result lines
and `artifacts/run/results.jsonl` for the structured suite source of truth.
