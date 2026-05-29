# Task 63 HNSW RaBitQ M5 Suite Config

## Summary

This packet adds a checked-in m5 laptop SuiteConfig for the Task 63 HNSW
RaBitQ benchmark gate:

- `benchmarks/task63-hnsw-rabitq-format/suite-m5.json`

The new config keeps the same measured matrix as the Linux/newer-Intel
`suite.json`: HNSW-only 50k/100k, TurboQuant/PqFastScan/RaBitQ, matched
`ef_search` sweep `40,64,100,128,160,200`, recall@10, latency, build/load
logs, and storage. It changes only host-specific execution details:

- M5 PostgreSQL socket path: `/Users/peter/.pgrx`
- M5 staged DBpedia fixture paths: `data/task31_m5_dbpedia_staged/...`
- M5 output isolation: `benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/`
- M5 precheck SQL avoids Linux `/proc` reads.

No benchmarks were run. This is a provenance/readiness change so the m5 laptop
can produce publishable Task 63 evidence without untracked SuiteConfig edits.

## Validation

Static validation only:

```sh
jq empty benchmarks/task63-hnsw-rabitq-format/suite-m5.json
jq -r '[.steps[].kind] | group_by(.)[] | "\(.[0]) \(length)"' benchmarks/task63-hnsw-rabitq-format/suite-m5.json
jq -r '.steps as $s | ($s | map(select((.kind != "raw") and (.name != "precheck-host"))) | length) as $measured | ($s | map(select((.kind != "raw") and (.name != "precheck-host") and ((.tags // []) | index("hnsw")))) | length) as $hnsw | "measured_steps=\($measured) measured_hnsw_tagged=\($hnsw)"' benchmarks/task63-hnsw-rabitq-format/suite-m5.json
jq -r '[.steps[] | .profile? // empty] | unique | .[]' benchmarks/task63-hnsw-rabitq-format/suite-m5.json
jq -r '[.steps[] | select(.sweep? != null) | .sweep[]] | unique | @csv' benchmarks/task63-hnsw-rabitq-format/suite-m5.json
rg -n "/proc|/var/run/postgresql|/var/lib/pgsql" benchmarks/task63-hnsw-rabitq-format/suite-m5.json
```

Results:

- JSON parsed cleanly.
- Step counts: `raw 1`, `load 6`, `recall 6`, `latency 6`, `storage 6`.
- Measured steps: `measured_steps=24 measured_hnsw_tagged=24`.
- Profiles: `ec_hnsw`.
- Sweep: `40,64,100,128,160,200`.
- No Linux `/proc`, `/var/run/postgresql`, or `/var/lib/pgsql` paths remain in
  `suite-m5.json`.
