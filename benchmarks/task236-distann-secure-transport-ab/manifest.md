# Task 236 DistANN secure-transport A/B

- Head SHA: `f63b67620933a48c7fe43f7bb4c63583b2342620`
- Task: `236`
- Packet: `benchmarks/task236-distann-secure-transport-ab/`
- Date: `2026-08-24`
- Lane: local PG18 release, three physical owners, real staged DBpedia corpus
- Storage / rerank: production RaBitQ neighbor codes, persisted 4096-sample
  head, `BW4/H100/L32`, lazy-10 materialization, no traversal replica
- Isolation: one freshly built index per table and one external PostgreSQL
  cluster per arm; no shared-table or fixture reuse surface
- A/B variable: `secure_remote_transport=false` (plaintext control) versus
  `secure_remote_transport=true` (TLS candidate). All other suite fields are
  identical within each scale.
- Suite config: `transport-ab-suite.json`, SHA-256
  `142257437332be069d878527ba485cddfd631305bc7a7354394ec3dd42fe9deb`
- Runner / extension provenance: the runner and all 18 PostgreSQL instances
  attest release extension SHA
  `f63b67620933a48c7fe43f7bb4c63583b2342620`.
- Run directories: `/home/peter/.ecaz/clusters/task236-{plaintext,tls}-{scale}`.
  These external directories were required because `distann-multicluster` has
  no run id and concurrent/per-arm runs need distinct paths. They were never
  review evidence and were removed after result capture.
- Build output: isolated external target
  `/home/peter/.ecaz/cargo-targets/task236-transport-ab`; no PostgreSQL data or
  benchmark output was written under repository `target/`.

## Commands

Audit:

```text
/home/peter/.ecaz/cargo-targets/task236-transport-ab/debug/ecaz bench suite audit --config benchmarks/task236-distann-secure-transport-ab/transport-ab-suite.json --log-file benchmarks/task236-distann-secure-transport-ab/artifacts/suite-audit.log
```

Each scale used the same command shape, selecting the plaintext and TLS steps
for that scale and writing a separate manifest/results pair:

```text
/home/peter/.ecaz/cargo-targets/task236-transport-ab/debug/ecaz bench suite run --config benchmarks/task236-distann-secure-transport-ab/transport-ab-suite.json --only plaintext-control-<scale> --only tls-candidate-<scale> --artifact-dir benchmarks/task236-distann-secure-transport-ab/artifacts/run-<scale> --manifest-output benchmarks/task236-distann-secure-transport-ab/artifacts/suite-manifest-<scale>.json --results-output benchmarks/task236-distann-secure-transport-ab/artifacts/results-<scale>.jsonl --log-file benchmarks/task236-distann-secure-transport-ab/artifacts/suite-run-<scale>.log
```

The three manifests each report `completed=2 failed=0`, with the four
unselected suite steps intentionally skipped and no missing or stale artifact.

## Result

| Scale | Recall plaintext / TLS | Warm mean plaintext -> TLS | p95 plaintext -> TLS | Physical generation plaintext / TLS |
| --- | --- | --- | --- | --- |
| 10k | 0.9990 / 0.9990 | 9.20 -> 8.03 ms (-12.72%) | 11.20 -> 8.90 ms | 242,860,032 / 242,860,032 B |
| 50k | 0.9545 / 0.9540 | 10.10 -> 10.20 ms (+0.99%) | 12.40 -> 11.80 ms | 1,243,512,832 / 1,243,488,256 B |
| 100k | 0.9275 / 0.9290 | 10.20 -> 9.99 ms (-2.06%) | 11.80 -> 11.20 ms | 2,498,215,936 / 2,498,199,552 B |

Secure transport introduces no material latency, recall, or storage regression
at 10k, 50k, or 100k. This single run does not claim TLS is intrinsically
faster: the small latency deltas change sign, and each arm independently builds
its physical graph. Recall deltas are at most 0.0015 and lie within the reported
overlapping 95% intervals. Storage deltas are zero at 10k and at most three
PostgreSQL pages at larger scales. The query SHA and head-membership SHA match
within every scale; all topology, remote-owner engagement, routed DML, and
NFR-021 rows pass.

The benchmark head also contains the Task 238 snapshot-lifetime fix in both
arms. It is not an A/B variable and prevents the pre-existing retry-path crash
from invalidating either measurement.

## Artifacts

- `artifacts/suite-manifest-{10k,50k,100k}.json`: authoritative suite state,
  expanded commands, config SHA, runner SHA, and step status.
- `artifacts/results-{10k,50k,100k}.jsonl`: normalized recall, warm latency,
  storage, provenance, head membership, engagement, and topology rows.
- `artifacts/run-<scale>/<arm>/distann-multinode-summary.log`: compact source
  rows parsed into each results file.
- `artifacts/run-<scale>/<arm>/physical-production-{recall,latency}.log` and
  `physical-production-predictions.json`: cited raw measurements and returned
  prediction sets.
- `artifacts/run-<scale>/<arm>/physical-head-membership.json`: matched head
  membership proof.
- `artifacts/report-{10k,50k,100k}.md`: suite-generated human-readable reports.
- `artifacts/suite-audit.log`, `suite-dry-run-10k.log`, and
  `suite-manifest-10k-dry-run.json`: pre-run shape validation.

No corpus TSV, truth cache, PostgreSQL cluster, tunnel/polling output, private
key, certificate, password, `sslkey`, `sslcert`, or `sslrootcert` value is
committed. A recursive case-insensitive scan of `artifacts/` for private-key
headers and those conninfo fields returned zero matches.
