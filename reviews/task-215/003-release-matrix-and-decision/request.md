# Task 215 review request: release A/B matrix and decision

Decision: STOP. BW64/H8 is not promoted; shipped defaults remain BW4/H100.

The complete normal PG18 release matrix passed all six arm harnesses, but the
candidate changed recall and was slower at every staged scale. See
`artifacts/decision.md` for the paired recall, latency-tail, storage, and
rollback evidence.

Please review `artifacts/task215-release-ab.json`, the generated suite
manifest/results, and the packet-local decision artifacts. The cited decision
run is under `artifacts/run-r2/`; the earlier stale-schema `artifacts/run/`
attempt is superseded and is not decision evidence.

The decision account now also includes
`artifacts/reconciliation-206.md`, which records why Task 206's roughly
194–231 ms BW64/H8 rows are not directly comparable to this packet's 22.6–31.6
ms rows: Task 206 used top-k 200/L200, while this release gate used top-k 10
and effective L64. The higher-recall/lower-latency trade is explicitly rejected
under the recall-equivalence clause, and the skipped standalone Task 208/210
entry-gate evidence is declared in `artifacts/decision.md` and the manifest.
The packet also explicitly records why mechanism counters were not added to
the uninstrumented release arms: Task 216 owns that separate diagnostic view,
and its feature-build latency is not release-decision evidence.
