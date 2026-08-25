---
task: 236
packet: 004-performance-closeout
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 236 performance and security closeout

Please review Task 236 for closeout at
`0ed7ffaebe749e193e1324ac3ce2624bfb12baf4`, using the implementation/security
evidence in packets 002/003 and the immutable benchmark packet at
`benchmarks/task236-distann-secure-transport-ab/`.

The release three-owner plaintext/TLS A/B completed at 10k, 50k, and 100k with
two successful arms per scale, no failed/missing/stale artifacts, matched query
and head-membership SHAs, and exact extension SHA attestation on every node.

| Scale | Recall plaintext / TLS | Warm mean plaintext -> TLS | p95 plaintext -> TLS | Physical bytes plaintext / TLS |
| --- | --- | --- | --- | --- |
| 10k | 0.9990 / 0.9990 | 9.20 -> 8.03 ms | 11.20 -> 8.90 ms | 242,860,032 / 242,860,032 |
| 50k | 0.9545 / 0.9540 | 10.10 -> 10.20 ms | 12.40 -> 11.80 ms | 1,243,512,832 / 1,243,488,256 |
| 100k | 0.9275 / 0.9290 | 10.20 -> 9.99 ms | 11.80 -> 11.20 ms | 2,498,215,936 / 2,498,199,552 |

The evidence supports a neutral conclusion: verify-full mutual TLS adds no
material recall, latency, tail, or storage regression at the required scales.
It does not claim TLS is faster; independent physical builds introduce small
run-to-run differences, all recall intervals overlap, and the largest storage
difference is three PostgreSQL pages.

Packet 003 separately proves TLS 1.3 mutual authentication, fail-closed CA,
hostname, certificate-validity, plaintext, and sslmode behavior, connection
reset/recovery, credential rotation with pool replacement/reuse, secure remote
DML, and zero secret exposure across live and artifact surfaces.

Please verify the benchmark A/B isolation and neutral interpretation, then
issue the outside security/code closeout verdict. The task status remains
review-open until that verdict is recorded.
