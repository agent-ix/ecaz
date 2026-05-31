---
task: 76
topic: closeout-preset-note
agent: codex
role: coder
model: GPT-5
date: 2026-05-31
---

# Task 76 Quality Preset Follow-Up

## Request

Please review this explicit closeout note for the Task 76 feedback in `reviews/task-76/002-closeout/feedback/2026-05-31-01-reviewer.md`.

## Decision

The quality-preset reloption is intentionally shelved for Task 76.

Task 76 measured defaults policy, not a new user-facing SPIRE tuning API. The measurement did not justify shipping a preset surface now:

- The measured 100k SPIRE balanced point is still expensive: tg32/nprobe32 was about `50.550 ms` p50 in the corrected Task 75 rerun while recall remained `0.9310`.
- The high-recall local point is about `132 ms` p50 at `0.9975` recall@10, while IVF nprobe96 reached `0.9980` recall@10 at `36.9 ms` p50.
- The canonical 1M fixture was unavailable, so a preset API would be based on 10k/100k local evidence and not the corpus-size coverage Task 76 wanted.
- Presets would create a durable reloption contract. Without 1M and AWS cross-host confirmation, that would freeze an API before the candidate-cost problem is understood.

The current default remains unchanged. If a SPIRE quality preset is still wanted after Task 77 reduces candidate/materialization cost or after 1M/AWS evidence exists, it should be reopened as a separate defaults/API task with explicit documentation and migration semantics.

## Status

Task 76 remains complete with no code change. This packet amends the closeout rationale only.
