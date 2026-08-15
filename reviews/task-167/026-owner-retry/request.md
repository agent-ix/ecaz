---
agent: codex
role: coder
model: GPT-5
date: 2026-08-15
seq: 1
---

# Task 167 owner retry follow-up

Status: review-open; NOT merge-ready. This packet records the fixes and the
remaining evidence gap; it does not request merge or task closeout.

The implementation fixes from the reviewer feedback are present in the pushed
production code:

- `RetainedGenerationScan::resolve_nodes` owns the exact missing-vec-id,
  intent-gated retry, with graph/directory guards reopened per attempt and a
  bounded wait outside the guards.
- Retry attribution is written to a separate unlogged side relation. The
  retry path does not update or lock the 2PC intent rows.
- The append-when-room diagnostic is a normal userset GUC and, as of
  `3c162f69d`, its value is honored by production `physical_dml` code.
- The fixture uses no mid-drill `ALTER TABLE`; the owner probe is indexed by
  `source_id`, large-scale latency uses five measured iterations plus two
  warmups, and the append A/B now uses five 32-row trials. Its pass condition
  requires both no throughput regression and no increase in backlink
  amendments; the output labels the arms as sequential on one fixture.
- The retry gate checks the vec_id carried by `OwnedRecordMissing`, rather than
  any id in the batch. Saturation requires the exact graph degree before and
  after the concurrent inserts.

The current production extension install now embeds `79afb0d82` from
`--release --no-default-features --features pg18`. Runtime preflight has not
completed because fresh external fixture initialization fails with
`Read-only file system`; the prior logs under this packet remain SHA
`563cb18f7` diagnostics. The latest harness checkpoint is `ac90e38a7`; the
latest product checkpoint is `79afb0d82`.

Existing production evidence is retained as diagnostic evidence, not current
head closeout evidence. The 10k run demonstrates an unforced natural retry
(`churn_retries=37`, `steady_retries=0`) but fails the full concurrency gate:
the real saturated target shrank from five to three neighbors, and only one
of the two controlled writer ids was present. Its inserted-neighborhood
fresh-rebuild recall is `0.133333`, so the old unconditional `pass=true` claim
has been removed from the packet's interpretation. The synthetic production
probe also shows natural retries (`99`) with no forced retry probe, but its
controlled target backlink check fails.

The required 50k/100k production matrix is not complete. The prior 50k/100k
attempts were either superseded by an invalid head cap or interrupted before a
decision-grade suite result. A fresh indexed rerun was blocked before setup
because the host root filesystem is read-only and the mandated external
cluster root cannot be recreated. No partial result is claimed as a pass.

Evidence and provenance are in `artifacts/manifest.md`. The task remains open
until the current production extension is installed, the 10k/50k/100k suite is
rerun with complete results, the unforced retry and concurrency gates pass, and
the low incremental-neighborhood recall is explained or fixed.
