# Task 205 L-bounded rerun manifest

This packet supersedes the inert Algorithm 1 measurement in
`reviews/task-205/003-ab/`. It measures the pushed implementation at the
required PG18 10k/50k/100k scales with fixed BW=4, H=100, graph degree 32,
head cap 4096, head width 32, head seed count 32, top-k 10, 200 queries, 50
iterations, 10 warmups, three physically sharded owner nodes, and no traversal
replica.

The current-code control uses `L=4096` as a non-binding reference arm. The
candidate uses `L=32`, the intended BW4/degree32 regime-sized default. The
short sweep uses `L=64`. The prior parent-build baseline remains the historical
baseline in `reviews/task-205/003-ab/`; it is not relabeled as this checkpoint.

## Provenance

- Task bucket/packet: `reviews/task-205/004-l-bounded-rerun/`
- Code/extension head: `0057a35c0461a8947612aab6b56d089eb67fa051`
- Extension: PG18 release build with `distann-head-attribution-benchmark`
- Runner: release `ecaz bench suite`; corrected run uses
  `artifacts/run-v2/` and `artifacts/suite-run-v2.log`
- Suite config SHA-256:
  `8b0582ef03904674de385b9e3e859b5c112c7b74d239bdd74c70f925f4882d3a`
- Config: `artifacts/task205-l-bounded-suite.json`
- Audit/dry-run: `artifacts/audit-v2.log`, `artifacts/dry-run-v2.log`
- Corrected fixture ports: 42120 through 42202; all run directories are
  under `/home/peter/.ecaz/clusters/`, outside the repository and Cargo
  target directory.

## Inputs

- Staged inputs: `/home/peter/dev/ecaz/data/staged-current`
- Corpus/query prefixes: `ec_real_10k`, `ec_real_50k`, `ec_real_100k`
- Corpus and query SHA-256 values are recorded in each packet-local
  `distann-multinode-summary.log` and suite `results.jsonl`.

## Evidence layout

- Structured result source: `artifacts/run-v2/results.jsonl`
- Suite provenance: `artifacts/run-v2/suite-manifest.json`
- Per-arm decision summaries:
  `artifacts/run-v2/{control-l4096,candidate-l32,sweep-l64}-{10k,50k,100k}/distann-multinode-summary.log`
- The suite runner derives and emits storage growth rows from the per-node
  storage rows. Those rows are measurement-only and carry
  `judgement=unjudged_nfr021_owner_resolution_pending`; this packet does not
  hardwire the disputed fixed-roster NFR-021 ratio gate.

The final request will cite the exact recall, latency, response/request-byte,
transport-wait, pruning-counter, and storage rows after all nine steps finish.
