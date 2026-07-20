---
task: 180
packet: 004-decision-rationale-correction
role: coder
status: open
date: 2026-07-15
head: 71690b6c4c06e4457c97a3254c6b6b53bea916d4
---

# Review request: correct Task 180 decision rationale

Task 180 remains a measured negative for widening the unchanged persisted head:
at 100k, width64/seeds64 produced 0.9280 recall versus production's 0.9275,
with overlapping confidence intervals, while p50 increased from 40.3 ms to
40.9 ms. That direction did not provide a useful relative improvement.

The original packet also called the proposed NFR-017 values hard gates. Those
numbers were agent-authored planning targets, not stakeholder-approved task
acceptance criteria. This packet corrects that rationale without changing the
measurements or reopening width/seed tuning. Task 181 remains the follow-up that
tested the distinct head-membership direction.
