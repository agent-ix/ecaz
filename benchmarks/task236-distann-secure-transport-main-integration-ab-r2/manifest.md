# Task 236 clean-main secure-transport A/B

- Head SHA: `48ea5d506c781ec92cfa91b0b756540f3b8cd8cd`
- Task: `236`
- Packet: `benchmarks/task236-distann-secure-transport-main-integration-ab-r2/`
- Date: `2026-08-24`
- Lane: local PG18 release, three physical owners, staged DBpedia corpus
- Storage / rerank: production RaBitQ neighbor codes, persisted 4096-sample
  head, `BW4/H100/L32`, lazy-10 materialization, no traversal replica
- Isolation: one freshly built index/table and one external PostgreSQL cluster
  per arm. No shared-table or fixture-reuse surface.
- A/B variable: `secure_remote_transport=false` (plaintext control) versus
  `secure_remote_transport=true` (verify-full mutual-TLS candidate). All other
  fields match within each scale.
- Suite config: `transport-ab-suite.json`, SHA-256
  `142257437332be069d878527ba485cddfd631305bc7a7354394ec3dd42fe9deb`
- Runner and every PostgreSQL node attest the exact, clean release extension
  SHA above. The earlier dirty-SHA attempt was discarded and is not evidence.
- Run directories were `/home/peter/.ecaz/clusters/task236-{plaintext,tls}-{scale}`.
  Separate external paths are required because `distann-multicluster` has no
  run id. They were removed after evidence capture; no PGDATA was under the
  repository or `target/`.

## Command

Each scale used the checked-in suite runner and the same command shape:

```text
/home/peter/.cargo-target/debug/ecaz bench suite run --config benchmarks/task236-distann-secure-transport-ab/transport-ab-suite.json --only plaintext-control-<scale> --only tls-candidate-<scale> --artifact-dir benchmarks/task236-distann-secure-transport-main-integration-ab-r2/artifacts/run-<scale> --manifest-output benchmarks/task236-distann-secure-transport-main-integration-ab-r2/artifacts/suite-manifest-<scale>.json --results-output benchmarks/task236-distann-secure-transport-main-integration-ab-r2/artifacts/results-<scale>.jsonl --log-file benchmarks/task236-distann-secure-transport-main-integration-ab-r2/artifacts/suite-run-<scale>.log
```

All three manifests report two succeeded selected arms, zero failed arms, and
four intentionally skipped steps.

## Results

| Scale | Recall plaintext / TLS | Warm mean plaintext -> TLS | p95 plaintext -> TLS | p99 plaintext -> TLS | Physical bytes plaintext / TLS |
| --- | --- | --- | --- | --- | --- |
| 10k | 0.9990 / 0.9990 | 8.51 -> 7.92 ms (-6.9%) | 9.40 -> 8.83 ms | 10.50 -> 9.60 ms | 242,860,032 / 242,860,032 |
| 50k | 0.9540 / 0.9545 | 8.86 -> 9.40 ms (+6.1%) | 10.50 -> 10.80 ms | 11.10 -> 12.60 ms | 1,243,512,832 / 1,243,504,640 |
| 100k | 0.9275 / 0.9295 | 9.16 -> 10.50 ms (+14.6%) | 10.80 -> 11.80 ms | 11.10 -> 12.20 ms | 2,498,215,936 / 2,498,215,936 |

Recall differences are at most 0.0020 and their reported 95% intervals
overlap. Storage is equal at 10k/100k and differs by one PostgreSQL page at
50k. TLS warm latency improved at 10k but increased at 50k and 100k, including
the tails. This packet makes no neutrality claim for latency and leaves the
mandatory-security tradeoff to outside review. Query SHA and head-membership
SHA match within each scale; topology, remote-owner engagement, and routed DML
checks pass.

## Artifacts

- `artifacts/suite-manifest-{10k,50k,100k}.json`: authoritative suite state,
  expanded commands, config SHA, runner SHA, and step status.
- `artifacts/results-{10k,50k,100k}.jsonl`: normalized recall, warm latency,
  storage, provenance, head-membership, engagement, and topology rows.
- `artifacts/run-<scale>/<arm>/distann-multinode-summary.log`: compact parsed
  source rows.
- `artifacts/run-<scale>/<arm>/physical-production-{recall,latency}.log` and
  `physical-production-predictions.json`: raw measurements and predictions.
- `artifacts/run-<scale>/<arm>/physical-head-membership.json`: matched-head
  proof.

No corpus TSV, truth cache, PostgreSQL cluster, node log, tunnel/polling output,
private key, certificate, password, or raw secret value is committed.
