# Review Request: C1 ADR-031 Real Corpus Recall Validation

## Context

Packet `281` landed the cached ADR-031 runtime path on `main`:

- cached binary codes on graph elements
- lazy exact scoring for newly loaded graph elements
- source-local ADR-031 successor gating on the ordered-scan runtime

Packet `282` then validated the warm steady-state latency result on the
normative real `50k` lane:

- `tqhnsw_real_50k`
- `m=8`
- `ef_search=40`
- `warm-after-prime3`
- `session-mode=per-cell`
- `timing-mode=cached-plan`
- `p50 = 4.633ms`
- `p99 = 7.661ms`

That clears the `NFR-001` latency target. The next risk is quality, not more
latency shaving.

## Problem

The cached ADR-031 path makes exact scoring lazy and adds a binary-sign
approximation inside successor handling. Even though the latency result is now
strong, we still need an explicit recall/quality read on the real corpus before
treating this runtime shape as a clean keep.

The next question is:

- does ADR-031 preserve the ordered-scan result quality at `m=8`,
  `ef_search=40`
- on the real corpus lane that now meets the latency target

## Planned Investigation

First step:

- inspect the existing real-corpus recall harness and confirm whether it can
  compare the current cached ADR-031 runtime path directly against exact truth
  or against the pre-ADR-031 ordered-scan surface

Preferred scope for the first bounded read:

- real corpus
- `m=8`
- `ef_search=40`
- enough queries to detect obvious regression before committing to a long run

If the existing harness already fits, use it. If not, add the minimum launcher
or harness seam needed to make the recall comparison explicit and repeatable.

## Success Criteria

- the packet records the concrete recall-validation command or harness used
- the packet records the first ADR-031 real-corpus recall comparison
- the result makes a clear keep/pivot call for the cached ADR-031 runtime path
