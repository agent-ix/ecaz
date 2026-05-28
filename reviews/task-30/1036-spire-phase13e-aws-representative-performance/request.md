# Review Request: AWS Representative Performance Attempt

Task: Phase 13e representative performance proof.

Current head: `2020771db`

## Summary

This packet records the first real representative AWS attempts against the DBpedia-derived `ec_real_100k` corpus. The representative measurement is not complete yet. The runs were still useful because they exposed two concrete blockers before p50/p95/p99 latency, recall, and pooling A/B evidence could be captured.

## Evidence

- `artifacts/run-representative-performance-pass-rerun-after-prepare-path-fix.log` reached real representative load/build and surfaced an oversized SPIRE manifest/object page payload failure.
- Commit `1202150cb` fixed the manifest page-overflow path with chained manifest blob storage.
- `artifacts/spire-large-manifest-blob-pg18.log` proves the local PG18 regression for the page-overflow fix:
  `spire_large_manifest_blob_pg18_pass object_count=257 placement_count=257 indexed_rows=10`
- `artifacts/run-representative-performance-pass-rerun-after-manifest-chain.log` reran the representative AWS pass after the page fix, but failed earlier than SPIRE build/query while streaming coordinator corpus load through the SSM PostgreSQL tunnel:
  `COPY finish failed for ec_spire_aws_repr_1m_corpus`
- That rerun reported `teardown complete and Terraform state is clean`.

## Follow-Up

Commit `2020771db` moves representative bulk load to node-local SSM execution with S3-staged TSV inputs. That follow-up is packeted separately in `reviews/task-30/1038-spire-phase13e-node-local-representative-load/`.

## Remaining Gate

Phase 13e still requires a fresh Graviton representative run that reaches `ecaz bench suite` and produces:

- p50/p95/p99 latency
- recall at the suite-gated nprobe values
- pooled-vs-unpooled production read profile deltas
- accepted representative summary verification

