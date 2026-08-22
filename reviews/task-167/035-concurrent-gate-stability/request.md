---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 concurrent-gate stability

Status: review-open; clean-head synthetic runtime pending.

Please review harness checkpoint `d799dc4fe`.

Packet 034 verified the snapshot-lifetime remediation through both full
remote-owner proofs, then exposed three fixture problems before the concurrent
2PC gate could be decision-grade:

- the synthetic inner-product corpus was not unit-normalized despite the
  extension's explicit build precondition;
- an extra target-seed insert consumed one of the two backlink slots reserved
  for two controlled writers;
- scanners and writers had separate start timing, so the fixed 12-query reader
  loops could complete before writer commit callbacks opened the visibility
  window the natural-retry assertion is meant to observe.

This checkpoint unit-normalizes only the synthetic physical fixture and its
later drill vectors, removes the extra target-seed insert, and starts all four
scanners plus both writers on one barrier. Each scanner now continues until
both writers finish, subject to a bounded 16x query budget; it still performs
at least the original 12 queries. Writer completion is published through an
atomic count.

The two focused Task 167 CLI tests pass. Runtime acceptance remains open until
the release CLI is rebuilt from a clean worktree, the matching production PG18
extension is installed, and the fresh synthetic suite gate passes.

