---
task: 229
packet: 004-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-28
seq: 01
---

# Task 229 full-scale decision — STOP

Review the Task 229 implementation at source head `6e4cb3c4a`, the release
PG18 installation recorded at packet head `76d283b3a`, and the completed
packet-local decision evidence. The checked-in twelve-step suite ran from
2026-08-27 13:18 PDT through 2026-08-28 00:24 PDT: 12 succeeded, 0 failed, 0
skipped, 0 missing artifacts, and 0 stale artifacts. Suite audit passes.

The explicit decision is **STOP**. The sidecar is semantically neutral and
distribution-conforming, but it is slower in both independent 100k
same-generation pairs. That uncontaminated primary read gate is the sole STOP
basis. It must not become the production default. This disposition does not
skip Tasks 230--232.

This revision addresses reviewer seq-12. The prior request incorrectly attributed
the cross-step build, total-storage, and DML deltas to the sidecar. All cover
steps, and no no-cover steps, enabled `materialization_correctness`; that
fixture adds an external uncompressed 12.8 KB `payload_note` to half the source
rows. Those three cross-step comparisons are therefore confounded and are not
decision evidence for or against the sidecar. No rerun is required because the
within-step primary read comparisons use the same covered generation and are
unaffected by the fixture difference.

## Primary read decision

Each row compares the forced row-tier control and sidecar candidate against
the same covered generation. Positive deltas are regressions.

| Scale | Pair | Control mean | Sidecar mean | Mean delta | p95 delta | p99 delta | Gate |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 10k | A | 11.2 ms | 11.7 ms | +4.46%, +0.5 ms | +5.69% | +1.46% | FAIL mean and p95 |
| 10k | B | 11.3 ms | 11.4 ms | +0.88%, +0.1 ms | 0.00% | -1.49% | pass local guard only |
| 50k | A | 13.0 ms | 13.3 ms | +2.31%, +0.3 ms | +5.33% | +5.84% | FAIL mean, p95, p99 |
| 50k | B | 13.9 ms | 14.2 ms | +2.16%, +0.3 ms | +2.41% | +2.92% | FAIL mean |
| 100k | A | 12.9 ms | 20.0 ms | +55.04%, +7.1 ms | +57.14% | +60.38% | FAIL primary and tails |
| 100k | B | 12.6 ms | 14.3 ms | +13.49%, +1.7 ms | +11.92% | +13.55% | FAIL primary and tails |

PROMOTE required both 100k pairs to improve by at least 5.0% and 0.50 ms.
Both instead regress. There is no averaging tie-break. All six
same-generation ordered prediction files are byte-identical, so the latency
result is not purchased with a recall or result change.

The attribution proves that the candidate actually selected the sidecar. At
100k it returned 3.34 local and 6.66 remote sidecar rows per scan while the
forced controls returned zero sidecar rows. Both mechanisms requested exactly
2.0 remote owners and 9.66 pushdown rounds per scan, so the candidate added no
remote round trip. Its owner-side sidecar lookup replaced row-tier payload SQL
but increased end-to-end time.

## Supporting measurements and confound correction

- The explicit sidecar heap+index is small: 0.41--0.42% of matched no-cover
  total bytes at 10k, 0.35% at 50k, and 0.34% at 100k, inside the 5% gate.
  This within-cover-arm relation measurement is clean and the storage gate
  passes. The observed 28.0--28.9% total-generation difference is not a
  sidecar cost: it is explained by the cover-only correctness fixture's
  64/320/640 MB of injected external payload at 10k/50k/100k plus a consistent
  8% TOAST/page overhead. It is excluded from the decision.
- NFR-021 passes for all six registered roles: decision eligible, conforming,
  no coordinator-resident unsharded bytes, no non-owned records or orphans,
  and maximum normalized bytes/owned-record growth 1.094499 against 2.0.
- The recorded build and DML deltas remain useful only as fixture observations.
  Because the cover steps alone carry the injected TOAST payload, the 10k
  build-ceiling breach and the replacement/delete deltas cannot be attributed
  to the sidecar and are not independent STOP reasons.
- Every covered 160-row insert arm reports 160 graph rows, 160 row-tier rows,
  and 160 sidecar rows appended across all three owners; no-cover arms report
  zero sidecar rows. Replacement/delete distributions each contain 32 routed
  single-row samples.

## Carried review items

- `results.jsonl` contains exactly 40 `physical_benchmark_stage` and 52
  `physical_benchmark_materialization_work` rows for every one of the 18
  measured step/variant cells, plus 10 insert-work rows for each suite step.
  This closes packet-003 seq-10's runner-behavior carry-in.
- Packet 003's coordinator disappearance limitation remains disclosed rather
  than overstated: the suite validates ordinary uncovered/covered remote reads,
  explicit sidecar selection, corruption, and DML, but does not delete a remote
  row-tier version between traversal and coordinator materialization. The
  uncovered-error/covered-skip branch restored by `6e4cb3c4a` therefore remains
  statically reviewed rather than dynamically injected.

## Provenance and artifact discipline

- All twelve backends passed the unanimous release preflight with extension
  SHA `76d283b3a309286877db0ffec30469ffae0ef6c1`, build profile `release`, and
  features `pg18,distann-head-attribution-benchmark`. The release install was
  performed after packet-003's debug `pg_test` installs, so it is the library
  measured by every latency arm.
- The suite-driving CLI is the shared-target debug binary. Reviewer seq-10
  cleared this because the timed region is only the PostgreSQL query round
  trip; harness decoding and statistics are outside it.
- Config SHA-256 is
  `2a55ad9ff85fbb796c2a7c6747b35161ee127db85f89b895980e343c8392b30c`;
  the manifest records runner commit `8bbc297948f83b7566c30c3c995e26ec7776975c`.
- The suite removed all twelve explicit `/home/peter/.ecaz/clusters/task229-*`
  run directories after durable capture. No custom Cargo target, repository
  PGDATA, new worktree, corpus copy, or truth cache is retained.
- Packet artifacts are pruned to 31 decision-grade files: the normalized
  16,540-row `run/results.jsonl`, suite manifest, 24 manifest-declared per-arm
  artifacts, audit/status/console logs, release-install log, and this manifest.

## Review request

1. Confirm the uncontaminated same-generation read evidence supports STOP
   under the preregistered primary rule, without relying on the confounded
   build/total-storage/DML comparisons.
2. Confirm the 40-stage/52-work-row and DML-distribution carry-in is closed.
3. Confirm seq-12's attribution correction is complete, packet 004 is DONE,
   and Task 229 may be tracking-closed and landed, retaining only the
   explicitly disclosed coordinator-injection limitation.
