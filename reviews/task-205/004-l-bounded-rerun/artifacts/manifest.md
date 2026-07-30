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
- Raw measurement code/extension head: `0057a35c0461a8947612aab6b56d089eb67fa051`
- Follow-up parser/assertion checkpoint: `045ce69e7` (release runner rebuilt
  after the raw arm measurements completed)
- Extension: PG18 release build with `distann-head-attribution-benchmark`
- Runner: release `ecaz bench suite`; raw arm run uses `artifacts/run-v2/`
  and `artifacts/suite-run-v2.log`; the follow-up uses
  `artifacts/suite-run-postprocess.log`
- Final suite config SHA-256:
  `17e8e01bbca2b77e97266cbc05bc1cf0e36ec0bd9043b26eb73fdf04408fb1a6`
- The raw run was started from the earlier config SHA
  `8b0582ef03904674de385b9e3e859b5c112c7b74d239bdd74c70f925f4882d3a`.
  After all nine arms succeeded, the current runner resumed the successful
  manifest with the final config and regenerated the derived rows/report; no
  benchmark arm was rerun during that normalization.
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
  measurement-only NFR-021 `context` registrations. The suite reports
  normalized bytes-per-owned-record conformance; it does not make any L arm
  decision-bearing or hardwire the disputed raw fixed-roster ratio gate.

## Final artifact inventory

The final manifest has 9/9 succeeded steps. `run-v2/results.jsonl` has 828
rows, including 9 `physical_benchmark_storage_ratio` rows, 9
`physical_benchmark_storage_growth` rows, and 3
`physical_benchmark_nfr_021_conformance` rows. The three NFR rows are
`task205-l4096-control`, `task205-l32-candidate`, and `task205-l64-sweep`;
each reports `actual_admissibility=conforming`, complete evidence, matching
registration, all three scales, normalized growth about 1.095, and raw
fixed-roster growth about 11.12. The raw growth rows are explicitly labelled
`reported_not_threshold_fixed_roster`; they are retained for diagnosis and are
not a decision-bearing NFR-021 gate.

| artifact | SHA-256 | purpose |
|---|---|---|
| `run-v2/results.jsonl` | `6f990c9bf7218d15a42775fe3cd2fc23345404551175911dc42f5494a9e0dd7c` | structured recall, latency, transport, counter, storage, growth, and NFR rows |
| `run-v2/suite-manifest.json` | `495e804aa505b9f11727e67848d41ac459208ee63516ac9412a7672a66b266ba` | final resumed manifest and provenance |
| `report-v2.md` | `eb4b16570795ed693bfeec817855b1400a721a7bbc4e12716ef8e226318fea0b` | current-runner report over the final structured rows |
| `suite-run-v2.log` | — | raw nine-arm suite execution log |
| `suite-run-postprocess.log` | — | resume/postprocessing log; all steps were already successful |

The exact recall, latency, response/request-byte, transport-wait,
pruning-counter, and storage rows are cited in the packet request. No corpus
TSV or cluster data is committed. The external task205 run directories under
`/home/peter/.ecaz/clusters/` are disposable and are removed after this
evidence capture.
