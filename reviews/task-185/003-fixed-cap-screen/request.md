---
task: 185
packet: 003-fixed-cap-screen
role: coder
date: 2026-07-23
head: c83ea6ea8426df0ae5ddc4e8dec55f68db801a94
status: review_requested
---

# Review request: fixed-cap 100k screen

Packet 002 accepted the benchmark-only gateway and basin policies with three
advisories. This packet carries all three explicitly and supplies the
pre-registered suite-only 100k A/B.

## Outcome

No Task 185 candidate is useful.

- Gateway membership has Jaccard 1.0 with the 4,096-row frequency control.
  The persisted order changed, but exact scoring returned no recall change:
  both exact arms are 0.9625 (95% CI 0.9532--0.9700).
- Basin diversification changed every query's returned seeds/order but reduced
  already-near-total basin overlap by only about 0.04 percentage point.
  Recall remained 0.9625 while warm mean latency rose from 20.40 to 66.10 ms
  on frequency and 19.80 to 67.40 ms on gateway.
- Storage is identical by construction. Gateway construction took 985,165 ms
  versus 931,189 ms for frequency and added only a different-order head plus
  benchmark diagnostics.

The 0.6 ms warm-mean difference between the two exact builds is not selected
as a latency win: they have identical recall, membership, query work, storage,
and extension code, and were measured in separate long-running builds.

## Interpretation

The isolated single-seed attribution found broad reachability, but it did not
transfer into the joint 32-seed beam: set cover ultimately selected the same
membership as control. The diversity selector attacked a measured redundancy
that was too small to matter and imposed a large query-time cost.

Selection is based solely on the held-out evaluation rows 1--200. Training and
validation diagnostics are explanatory only. The unequal tail-fill policy and
isolated-budget upper-bound semantics are recorded in the artifact manifest.

## Validation and provenance

The PG18 release-profile suite completed both exact/disjoint three-owner
steps, 200 queries / 2,000 trials, and 50 warm samples after 10 warmups. It
reports zero failures, skips, missing artifacts, or stale artifacts. A focused
CLI test target passes 12 tests after the diagnostic aggregation was bounded
to one query per PostgreSQL statement. Full provenance, commands, checksums,
topology, storage, and the excluded pre-measurement attempts are in
[`artifacts/manifest.md`](artifacts/manifest.md).

## Review questions

1. Does the evidence support rejecting both gateway membership and basin
   diversification at fixed cap 4,096?
2. Is it correct to treat the exact-arm latency difference as noise rather
   than a candidate win?
3. Does the identical membership plus flat evaluation recall satisfy Task
   185's conditional rule for skipping the 10k/50k/100k confirmation?
4. Is the resulting STOP and handoff to Task 186's isolated cap-8,192
   capacity control decision-grade?
