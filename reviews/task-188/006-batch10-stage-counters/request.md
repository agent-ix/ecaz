---
task: 188
packet: 006-batch10-stage-counters
role: coder
status: open
date: 2026-07-27
head: 193cff682
---

# Review request: normalize paired recall and qualify batch-10 latency mechanism

This follow-up addresses the accepted packet-005 corrections:

- the suite parser now preserves `physical_benchmark_paired_recall` rows in
  structured `results.jsonl`, with a focused parser regression test;
- packet 005's existing results were re-normalized, producing all three paired
  rows with the accepted win/loss/tie and bootstrap values; and
- the batch-10 latency inversion is explained using the available instrumented
  hop-round/remote-candidate attribution, with the eager-0 limitation stated
  explicitly.

A fresh explicit-batch-10 stage-counter diagnostic was preregistered and
attempted, but the 100k physical build failed with `ENOSPC`. No values from the
failed run are used as evidence. The packet-local diagnostic note records the
failure and the exact fallback qualification.

See `artifacts/stage-counter-diagnostic.md` and packet 005's updated manifest.

The follow-up efficient diagnostic subsequently completed at code checkpoint
`193cff682`. Its packet-local evidence is under
`artifacts/run/efficient-20260727-r2/`; the direct batch-10 stage attribution
is summarized in `outcome.md`. Recall/storage interpretation remains tied to
packet 005 because this targeted rerun deliberately omitted the duplicate
recall/storage matrix.
