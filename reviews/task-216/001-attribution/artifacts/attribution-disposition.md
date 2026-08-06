# Task 216 attribution disposition

## Run identity

- Fresh physical generation: `ec_real_100k`, three sharded owners, no full
  graph replica, no traversal replica, build shards 1.
- Control: shipped BW4/H100 behavior, effective L32, persisted-head seeds 32,
  graph degree 32, top-k 10, 50 latency iterations and 10 warmups.
- Diagnostic build: PG18 release with
  `distann-head-attribution-benchmark`; it was installed before the suite and
  the checked-in config used `skip_install=true`.
- Extension commit: `c09c9113f3f48dcbd0f5acf7c3b8a96888d00764`.
- Config SHA-256:
  `1d47d75edd8352973e39e69327a5e6929b97321e6be4621119fc088b543a6cb8`.

## Decision-quality result

The topology gate passed: 100,000 source rows, three owners, zero non-owned
records, zero orphan vectors, two verified remote owners, and no unexpected
coordinator-derived storage. Physical recall was 0.9290 (95% CI
0.9169–0.9395). Feature-build latency is diagnostic and is not comparable to
the normal release A/B matrix: mean 39.30 ms, p50 38.50 ms, p95 50.30 ms,
p99 54.70 ms, max 55.60 ms.

The full-metrics stage decomposition, in milliseconds per scan, was:

Denominator note: `custom_scan_total`, `traversal_total`, and the other
scan-level rows are per-scan totals. The rows whose counters have
`samples=100`—including `materialize_owner_endpoint_work`,
`materialize_owner_payload_sql_work`, and `materialize_request_roundtrip_work`—
sum wall time from two remote owners across 50 scans. Their corresponding
per-owner means are approximately 20.291, 19.690, and 23.053 ms/owner/scan;
the summed values are retained because they describe total remote owner work,
but must not be read as a single-owner or additive whole-scan duration.

| Stage | Mean ms |
| --- | ---: |
| `custom_scan_total` | 37.208 |
| `materialize_owner_endpoint_work` | 40.583 |
| `materialize_owner_payload_sql_work` | 39.381 |
| `materialize_request_roundtrip_work` | 46.105 |
| `remote_materialize` | 25.958 |
| `traversal_total` | 8.214 |
| `traversal_transport_wait` | 4.100 |
| `traversal_owner_service` | 2.338 |
| `traversal_owner_graph_read` | 1.221 |
| `traversal_owner_score` | 0.966 |
| `materialize_coordinator_decode` | 0.075 |
| `traversal_owner_response_encode` | 0.002 |

Work counters were 579,163 payload bytes returned per scan, 14,135
traversal request bytes, 4,319 traversal response bytes, 31.34 remote rows
returned, and 10 total executor rows consumed. Backend RSS samples rose from
131,436 KiB to a 144,156 KiB maximum over 74 samples; this is diagnostic
memory evidence, not a leak conclusion.

## Candidate screen

- `MAT-15` remains eligible and is the strongest next hypothesis: packed
  payload buffers with offsets/null bitmap target the dominant owner payload
  SQL/endpoint materialization work.
- `MAT-21` remains eligible as a secondary hypothesis: typed/binary locators
  target owner locator formatting inside the same materialization region.
- `TRAV-05` is rejected for this screen: owner response encoding measured
  0.002 ms/scan, so packed expansion responses do not target the dominant
  stage in this control.

No candidate was implemented or advanced to A/B in this packet. Any follow-up
must choose at most one of `MAT-15` or `MAT-21`, preserve the Task 205
threshold/L semantics and FR-079 ordering, and run an isolated 100k A/B before
any 10k/50k/100k release matrix.

## NFR-021 runner caveat

The physical topology gate passed, but the suite-derived NFR-021 growth row was
`actual_admissibility=unavailable` because this diagnostic intentionally has a
single 100k scale and cannot establish the multi-scale normalized-growth
check. Therefore the suite process returned nonzero at its final
registration assertion. This is the same one-scale diagnostic limitation
recorded in the accepted Task 206 attribution packet; it is not treated as a
conforming release decision arm or as evidence that an NFR violation exists.

The final installed extension was restored to the normal PG18 release build;
the attribution-only SQL entities are absent from the installed schema.
