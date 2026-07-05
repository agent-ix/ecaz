# Task 133: TQ IVF non-scorer latency profiling (attack the remaining ~40%)

Status: **proposed** (2026-07-01). Owner: coder (to be assigned). Priority: P2
Follow-up to Task 125.

## Why

Task 125's int16 LUT cut the TurboQuant kernel ~48% but IVF query latency only
~30% (10k: 1.26→0.88 ms; 100k: 3.86→2.73 ms). Back-of-envelope, the scorer was
therefore only ~60% of per-query time; **~40% is non-scorer work** — top-k heap
operations, page/posting I/O, candidate materialization, and payload decode.
Now that the big slice is 2× faster, Amdahl says the next win is increasingly in
that 40%. We do not currently have a per-stage attribution of IVF query latency.

## Scope

- Produce a flamegraph / per-stage attribution of IVF no-QJL 4-bit query latency
  at 10k/50k/100k (scorer vs heap/top-k vs I/O vs materialization vs decode).
- Use the existing profiler harness (packets 028/030 lineage) and/or in-engine
  stage timers; extend `ecaz bench` with stage counters if needed rather than
  ad hoc glue.
- Identify the top 1–2 non-scorer hotspots and propose targeted follow-up tasks
  (this task is measurement + direction, not the fix).

## Out of Scope (hard)

- No new on-disk format/mode. No speculative optimization without the profile
  first.

## Required Evidence

- A committed per-stage latency breakdown at 10k/50k/100k with the method
  recorded, stored packet-local.

## Gate / Exit Criteria

- A source-grounded attribution of where the non-scorer ~40% goes, plus a ranked
  shortlist of follow-up optimization targets (with rough expected ceiling per
  Amdahl). Closes when the breakdown + shortlist land.
