# Task 67 Review Request: bf16 SQL decision

## Summary

This packet provides the Slice I bf16 on/off SQL decision measurement on the AWS Intel `10k-intel` lane.

Result: do not enable `rabitq-bf16` by default for this lane. The bf16-enabled build preserved recall but was slower than the non-bf16 build at all tested nprobe values.

## Measurement

Both runs used `ecaz bench suite` with checked-in SuiteConfig files:

- `artifacts/task67-bf16-off-suite.json`
- `artifacts/task67-bf16-on-suite.json`

Fixture and index shape:

- `ec_real_10k`, 200 queries, PG18
- `storage_format=rabitq`
- `quant_bits=4`
- `rerank=heap_f32`
- `rerank_width=100`
- nprobe sweep: `16, 32, 64`
- isolated prefixes: `task67_bf16off_10k_rabitq4`, `task67_bf16on_10k_rabitq4`

## Results

| nprobe | bf16 off p50 | bf16 on p50 | decision |
| ---: | ---: | ---: | --- |
| 16 | 2.02 ms | 2.25 ms | bf16 on is 1.11x slower |
| 32 | 3.32 ms | 3.58 ms | bf16 on is 1.08x slower |
| 64 | 5.52 ms | 6.45 ms | bf16 on is 1.17x slower |

Recall was equal at the reported precision:

- nprobe 16: both `0.9985`
- nprobe 32: both `1.0000`
- nprobe 64: both `1.0000`

See `artifacts/comparison.md` for p50, mean, recall, and mean q-time.

## Validation / Artifacts

See `artifacts/manifest.md` for commands, S3 run URIs, recovery logs, and key result lines.

Successful benchmark logs:

- bf16 off: `artifacts/bf16-off/cloud-bench-bf16-off.log`
- bf16 on: `artifacts/bf16-on/cloud-bench-bf16-on-rerun-after-corpus-restore.log`

AWS was paused after the successful bf16-on run; `artifacts/preflight/cloud-status-after-bf16-on-success.log` records `$0.00/hr running` with the instances stopping.

## Notes

The packet includes failed preflight attempts because they explain the support changes in packets 030-032:

- skip CLI rebuild after the bf16 extension build filled the disk during an unnecessary CLI rebuild
- clean Cargo target before git reset after the host was too full for git index writes
- restore the staged 10k corpus after `cargo clean` removed `target/real-corpus`
