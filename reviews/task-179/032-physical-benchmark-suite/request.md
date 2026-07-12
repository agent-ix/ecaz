---
task: 179
packet: 032-physical-benchmark-suite
role: coder
status: review-requested
head: 4570ac008
date: 2026-07-12
---

# Review request: physical benchmark suite surface

Please review code commit `4570ac008`.

This adds a reusable `physical_benchmark` mode to the existing
`distann-local-multinode` suite step. After the physical topology gate succeeds,
the fixture:

- preserves the published physical control table under a standard benchmark prefix;
- builds a same-data, same-commit single-instance ec_distann control;
- invokes the standard `ecaz bench recall` and `ecaz bench latency` commands for
  both arms;
- reuses one exact truth cache outside the committed packet;
- emits normalized recall, latency, build-time, storage, and remote-engagement rows;
- records cluster-total physical generation bytes separately from coordinator source
  bytes and single-instance index/source bytes; and
- declares all four standard benchmark logs as suite artifacts.

The real-corpus dimension now comes from the staged manifest instead of the
synthetic `--dim` default.

Focused validation is in `artifacts/validation.log`. A release 10k development
probe (not promoted as evidence) established that the standard prepared query
uses the physical CustomScan: recall@10 was 1.0000 on both arms, with mean query
time 10727.91 ms physical versus 1043.92 ms single-instance. The scale matrix
will run only after this checkpoint is committed and pushed.

Review focus:

- identifier/path derivation from the validated corpus prefix;
- same-data A/B construction and truth-cache reuse;
- table-output parsing for recall and latency;
- physical cluster storage accounting; and
- suite expected-artifact and normalized-row coverage.

