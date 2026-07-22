---
task: 194
packet: 008-nine-way-completion-audit
role: coder
status: review_requested
date: 2026-07-22
seq: 1
---

# Task 194 nine-way completion-audit repair

The completion audit reopened Task 194 because packets 006/007 did not satisfy
the written Phase 1 contract. Remote responses exposed only total owner service
and open/validate; the graph-read/score stage rows came from coordinator-local
expansion. No traversal connection/prepared-state, query-cache, request/response
byte, remote response-encode, or client receive/decode rows existed.

Commit `1b5e201a9` completes the benchmark-feature-only attribution:

- remote owners return open/validate, graph lookup, scoring, response-row
  assembly, logical response bytes, and total service sidebands;
- the coordinator records pooled connection and prepared-statement work,
  query-cache hits/misses, logical request bytes, critical RPC wall, client
  receive/decode, transport residual, and owner-service straggler spread;
- local graph/score timers no longer masquerade as remote owner work; and
- the fixture fails unless the remote decomposition reconciles within 5% of
  `remote_expand` and the traversal decomposition within 10% of
  `traversal_total`.

Normal PG18 builds retain the production SQL ABI and passed strict clippy with
warnings denied. The attribution-feature build and focused reconciliation
parser test also pass.

The rerun copies the existing canonical config into packet 008 with only
artifact/run paths, base port, and identifying labels changed, preserving the
immutable packet-002 evidence.
Artifact provenance and results will be recorded in `artifacts/manifest.md`.

## Canonical rerun result

The corrected release suite succeeded at 100k with all 34 stage rows and 26
work rows present. Recall remained `0.9625`; warm latency was `27.70 ms` mean
and `34.10 ms` p95. The accounting gates passed:

- remote expansion: `7.429284 ms/scan`, decomposed to `7.342625 ms/scan`
  (`1.17%` error, `5%` tolerance);
- traversal total: `9.065098 ms/scan`, decomposed to `8.945684 ms/scan`
  (`1.32%` error, `10%` tolerance).

The completed owner/transport split confirms the same candidate selection as
packet 006. Remote owner service is `2.258880 ms/scan`, while transport wait is
`5.012911 ms/scan`; connection readiness, request encoding, and coordinator
receive/decode total only `0.070834 ms/scan`. There are 10 sequential hop
rounds, 40 requested/returned nodes, and zero repeated nodes per scan. Logical
request/response volume is 13,871.92 / 10,530.32 bytes per scan.

Packet 007 already measured the selected fixed-work wider/fewer-round candidate
as a paired A/B on one shared generation. Its STOP remains valid: BW=8/H=50
reduced rounds and transport wait and improved recall, but did not improve
end-to-end mean usefully and regressed p95. The corrected canonical attribution
does not select a different candidate family, so no second candidate or
duplicate A/B is warranted. Task 194 closes STOP with production behavior
unchanged.

The owner timers and clock reads are compiled only under the attribution
feature. Row-side telemetry cannot represent an owner response containing zero
rows; such a response is omitted from critical-owner/straggler aggregation.
This healthy run did not encounter missing returns (`40.0` nodes requested and
`40.0` returned per scan), but failure/empty-response attribution must use a
future out-of-band response envelope rather than treating this packet as proof
for that case. Task 197 records the reviewer's separate fail-fast release-build
preflight recommendation.
