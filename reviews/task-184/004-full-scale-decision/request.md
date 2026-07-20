---
task: 184
packet: 004-full-scale-decision
role: coder
status: open
date: 2026-07-19
head: 6d4e1da825c9e9d0bd4f84f0bbc66ec4685afcfe
---

# Review request: full-scale materialization decision

## Outcome: PROMOTE to a productionization follow-up

Promote the fixed batch-10, executor-driven payload materialization design from
its benchmark-only implementation to a separately reviewed production task.
Do not flip the production path in Task 184: normal builds remain eager until
that follow-up lands the normative contract and production implementation.

The checked-in suite completed matched eager/lazy10 cells at 10k, 50k, and
100k. Each pair shared one immutable physical generation and the same persisted
head/seed digest. Distinct recall was byte-for-byte identical while the lazy10
arm reduced warm mean latency by 38.3% to 41.5% and improved every reported
tail statistic.

| Scale | Recall eager / lazy10 | Mean ms eager / lazy10 | p50 | p95 | p99 | Max |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9990 / 0.9990 | 34.10 / 20.70 (-39.3%) | 34.10 / 20.40 | 40.10 / 23.80 | 43.20 / 25.00 | 44.70 / 25.30 |
| 50k | 0.9685 / 0.9685 | 36.00 / 22.20 (-38.3%) | 35.00 / 21.90 | 42.60 / 24.90 | 49.30 / 25.90 | 51.20 / 26.80 |
| 100k | 0.9625 / 0.9625 | 38.30 / 22.40 (-41.5%) | 38.30 / 22.20 | 48.40 / 25.60 | 52.90 / 26.30 | 54.40 / 26.80 |

The attributed target moved consistently. Remote materialization fell from
22.497/24.244/25.596 ms to 9.901/10.037/10.179 ms at 10k/50k/100k. Remote
candidates requested fell from 23.68/26.36/26.84 per query to
6.58/6.72/6.64, and payload bytes fell from 437,606/487,133/496,003 to
121,598/124,186/122,707 bytes per query. The executor consumed the same rows;
the eager arm had fetched ranked remote candidates it never consumed.

Storage and construction are shared within every A/B pair: physical generation
bytes are 242,745,344 / 1,242,734,592 / 2,496,659,456 and physical build times
are 70,971 / 398,792 / 854,786 ms at 10k/50k/100k. Every cell passed the
three-owner physical-topology and two-remote-owner engagement gates, query
separation, and unanimous clean installed-extension provenance at
`765f28a548b194f1bd1ba845ae06b2266d04153b` (release).

Packet 003's adversarial semantic matrix remains the correctness gate: exact
eager/lazy output identity for unfiltered results, filters rejecting the first
and multiple batches, null and wide compressed-inline varlena projection/qual
values, mixed
local/remote winners, and a post-first-batch owner outage. All seven cases
passed; later remote failure still fails the query rather than returning a
partial result. Work remains bounded by the already-ranked candidate set and
deepens in deterministic global rank windows of 10. Each request ends at the
current proven prefix and total qual-driven payload reads retain the existing
corpus-independent ceiling fixed once from the initial search bar
(`max(initial × 64, 1024)`). ADR-085 D12 records this exact choice. Task 191
must reconcile NFR-019's older unconditional `payload reads <= k` prose before
making D12 the production default; the bound itself is not left undecided.

No 1m cell ran because `data/staged-current/` contains only the attested 10k,
50k, and 100k corpora. This is the task's explicit provenance-conditioned 1m
skip, not a failed gate.

The suite manifest labels the runner `b51b0ad...-dirty` because the suite writes
its tracked output manifest before capturing runner state. The runner binary
was built at committed checkpoint `b51b0ad4795290731bf1b8044117701af6527c8a`;
more importantly, all three PostgreSQL nodes independently report the same
clean installed extension SHA and release profile above.

Review the compact evidence in `artifacts/manifest.md`, the 279 structured
rows in `artifacts/full-run/results.jsonl`, and the three scale summaries.
Decision checkpoint `98ff482f6209d1da2f12bd1ebc3d20ed4861ae69`
records ADR-085 D12, closes the Task 184 roadmap entry, creates Task 191, and
gates Task 187 on the productionized baseline.

## Outside review: ACCEPT

The outside reviewer accepted packets 001--004 and the PROMOTE decision on
2026-07-20. No blocking code, correctness, evidence, or decision finding
remains. Four non-blocking technical observations are acknowledged in
`artifacts/review-disposition.md` and made explicit requirements of Task 191 at
checkpoint `6d4e1da825c9e9d0bd4f84f0bbc66ec4685afcfe`.

The coder has not edited the feedback or self-closed this request. The
reviewer's ACCEPT is the acceptance record.
