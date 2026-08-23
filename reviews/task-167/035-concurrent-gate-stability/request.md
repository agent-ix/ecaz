---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 concurrent-gate stability

Status: review-open; clean-head synthetic gate passed, isolated normalization
follow-up carried.

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

The two focused Task 167 CLI tests pass. The clean-head synthetic suite now
passes at `31e6c9138` with a release, production-feature (`pg18`) extension:
both controlled writer backlinks survive, the target fills from six to eight
neighbors, three natural 2PC-wave retries are attributed, steady-state retries
remain zero, and the complete physical topology gate passes.

The successful run still emits one unit-normalization warning from the
separate `mi` rollback subfixture, whose generator predates the normalized main
physical fixture. That warning did not fail the rollback gate, but it is being
removed before the real 10k/50k/100k matrix so every synthetic DistANN build in
the final checkpoint honors the same documented precondition.
