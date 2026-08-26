# Task 239 packet 002 one-shot decision

## Disposition

**ONE SHOT CONSUMED — PRE-SEMANTIC CLI/EXTENSION COUNTER-SCHEMA
INCOMPATIBILITY; NO PACKET-002 RERUN.**

The only live run authorized by reviewer seq02 was invoked once and exited 1
after 139,319 ms. It did not execute any of the nine semantic scenarios, so it
neither passes nor falsifies the corrected batch-state hypothesis.

## What passed

- Live suite manifest: runner `d03997c7aef2ff217d0535b47d0b8af765b8500f`,
  config SHA-256
  `bd74199c5fc26d7dffc6b72582915529cbd1c7453ec4ff8fdaad82d7605e6f21`,
  `dry_run=false`, one selected step.
- Extension preflight: unanimous release
  `41392c011106cb040095fd6004c4d5c0f136f1a0`, features
  `distann-head-attribution-benchmark,pg18`, no `pg-test`.
- Fresh 10k three-owner fixture setup and serving checks passed.
- The eager-control recall child completed at 0.9990 over 200 queries / 2,000
  trials. Its predictions SHA-256 is
  `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`,
  byte-identical to packet 001.

These facts establish provenance and partial setup only. There is no lazy-10
recall child, no summary/results JSONL, and no semantic decision evidence.

## Failure

The first eager-control latency child emitted all 37 stage rows exposed by the
exact-main extension, then the corrected CLI failed at
`distann_multicluster.rs:8433`:

```text
physical latency attribution expected 40 ec_distann stage rows
(1 concurrency groups), got 37
```

The mismatch is exact and pre-existing outside the Task 239 batch-state fix:

- exact main `41392c011` defines `DistannQueryStage::ALL` with 37 rows;
- the later Task 224 CLI checkpoint `d03997c7a` hard-codes 40 rows after adding
  `materialize_owner_payload_spi_work`,
  `materialize_owner_binary_send_work`, and
  `materialize_owner_response_construct_work`;
- seq02 correctly checked that the new stage labels were not needed by the
  requested Task 239 calculations, but missed the unconditional exact row-count
  assertion that runs before those calculations and before the semantic matrix.

This is a runner/extension compatibility failure. It is not evidence about
eager versus lazy bounded reads and does not authorize changing the bound.

## Run discipline and cleanup

- No `--continue-on-error`, resume, selected-step execution, replacement, or
  second attempt occurred.
- The main log contains zero `physical_materialization_correctness` rows.
- The harness stopped all three PostgreSQL nodes. The stopped 1.2 GB external
  run directory was removed after packet-local evidence was captured; it is
  recoverable only by regeneration.
- Packet 002 returns to outside review. Any further live evidence requires a
  separately reviewed compatibility correction and new authorization; this
  packet's one-shot authorization is exhausted.
