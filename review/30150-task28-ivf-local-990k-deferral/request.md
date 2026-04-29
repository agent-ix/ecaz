# Task 28 IVF Local 990k Recall Deferral

## Scope

Record the decision to stop treating fresh 990k exact-recall completion as a
local desktop merge blocker for Task 28.

This follows packet 30149, where the 990k width-250 recall attempt was stopped
after roughly 22 minutes while still fetching the full source matrix from local
PG18.

## Decision

Defer additional 990k exact-recall runs to a better-suited benchmark
environment. Local desktop validation should continue to cover:

- code correctness
- PG18 callback behavior
- 10k/25k/100k IVF tuning surfaces
- harness shape and artifact hygiene
- local smoke evidence for any new measurement plumbing

The existing 990k IVF packets remain useful as directional local evidence, but
fresh 990k recall gaps should not block landing when they are constrained by
desktop memory, storage, and source-matrix transfer cost rather than IVF code
correctness.

## Current Evidence

- 30130 established the current 990k IVF selected surface.
- 30132 and 30133 established the local 990k lower-`nprobe` frontier at
  `rerank_width=500`.
- 30136 showed `rerank_width=250` has modest latency upside but lacked recall.
- 30149 showed the attempted local recall fill is blocked by harness-scale
  source materialization on this machine.

## Consequence

Keep the harness improvements from 30147-30149:

- `--truth-cache-dir` for same-run / repeat-run exact truth reuse.
- partial top-k selection for first-run exact truth extraction.
- `--truth-cache-file` for explicit truth reuse that can skip full corpus fetch
  on cache hits.

Do not start more local 990k recall jobs for Task 28 unless there is a specific
new hypothesis being tested.

## Next Local Work

Use local time for the remaining landable IVF items:

- consolidate the current gate status after A3/A7/A10 packets
- make the 990k deferral visible in the Task 28 evidence map
- run only focused PG18 tests for code touched since the last packet
- keep any further measurements at 10k/25k/100k unless explicitly escalating to
  larger benchmark hardware

## Artifacts

- `artifacts/manifest.md`
