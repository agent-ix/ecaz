---
task: 236
packet: 005-main-integration
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 236 current-main final review

Please perform final security/code review of Task 236 at
`48ea5d506c781ec92cfa91b0b756540f3b8cd8cd`.

This is a clean reconstruction on current `main`, not the earlier branch
stacked on rejected Task 234. It excludes Task 234's production read
cancellation/await-finalize behavior. The only retained diagnostic addition in
that area is a `pg_test`-gated pool-census probe required by Task 236's
credential-rotation matrix; it does not alter production read finalization.

The implementation routes every production ec_distann PostgreSQL connection
through the shared secret-backed TLS connector, preserves verify-full and
channel-binding policy, restricts plaintext to permitted loopback endpoints,
keys pooled sessions by sanitized transport identity, and provides bounded
credential-rotation invalidation without exposing secret values. Packet 003
contains the PG18 TLS 1.3 mutual-authentication, fail-closed negative matrix,
reset/recovery, pool-rotation, remote-DML, and secret-exposure evidence.

Focused current-main validation passed: PG18 library check without warnings,
8 shared TLS connector tests, the secure-suite expansion test, and the
current-main Task 167 suite-regression preservation test. See
`artifacts/manifest.md`.

The authoritative exact-clean-SHA benchmark packet is
`benchmarks/task236-distann-secure-transport-main-integration-ab-r2/`:

| Scale | Recall plaintext / TLS | Warm mean plaintext -> TLS | p95 plaintext -> TLS | p99 plaintext -> TLS | Physical bytes plaintext / TLS |
| --- | --- | --- | --- | --- | --- |
| 10k | 0.9990 / 0.9990 | 8.51 -> 7.92 ms | 9.40 -> 8.83 ms | 10.50 -> 9.60 ms | 242,860,032 / 242,860,032 |
| 50k | 0.9540 / 0.9545 | 8.86 -> 9.40 ms | 10.50 -> 10.80 ms | 11.10 -> 12.60 ms | 1,243,512,832 / 1,243,504,640 |
| 100k | 0.9275 / 0.9295 | 9.16 -> 10.50 ms | 10.80 -> 11.80 ms | 11.10 -> 12.20 ms | 2,498,215,936 / 2,498,215,936 |

Recall intervals overlap and storage is neutral. TLS latency improves at 10k
but increases at 50k and 100k (100k mean +14.6%, p95 +9.3%, p99 +9.9%). This
request deliberately does not repeat the earlier packet's latency-neutral
claim. Please judge that measured cost against the mandatory P0 transport
security requirement and issue the outside closeout verdict.

No formatter was run. No corpus, PGDATA, operational node logs, certificate,
key, password, or secret value is included in the packet.
