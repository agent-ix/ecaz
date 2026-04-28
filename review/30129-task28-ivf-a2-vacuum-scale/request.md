# Task 28 IVF A2 Vacuum Scale Evidence

## Scope

This packet records the missing A2 scale measurement for the streaming IVF vacuum implementation.

The harness builds one 1M-row synthetic PG18 IVF surface per `nlists` value, deletes half the rows, runs `VACUUM (ANALYZE)`, samples backend memory during vacuum, and records vacuum wall time plus index size.

## Result

| nlists | rows before | rows after | vacuum ms | index before | index after vacuum | RSS peak KB | HWM peak KB | samples |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 8 | 1000000 | 500000 | 2359 | 89055232 | 89055232 | 364476 | 426708 | 88 |
| 32 | 1000000 | 500000 | 2047 | 89055232 | 89055232 | 370056 | 432476 | 77 |
| 64 | 1000000 | 500000 | 2029 | 89063424 | 89063424 | 370600 | 433096 | 75 |

## Interpretation

This closes the A2 evidence gap from the reviewer checklist for the requested `nlists in {8,32,64}` at >=1M rows.

The relevant vacuum-time memory signal is `rss_peak_kb`, sampled from `/proc/<pid>/status` while `VACUUM` was running. It stays in a narrow 364476-370600 KB band even as `nlists=8` creates much larger per-list posting lists than `nlists=64`, which supports the streaming-vacuum claim that memory is not proportional to the full posting-list length.

`hwm_peak_kb` is also recorded for completeness, but it is process lifetime `VmHWM` on the same backend and can include setup/build history before the vacuum window. Treat RSS peak as the cleaner per-vacuum signal in this packet.

Index size does not shrink in this harness because the test deletes half the rows but does not run a refill/churn phase. Space reuse and compaction behavior is covered separately in the A3 packets.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_a2_vacuum_scale --rows 1000000 --nlists 8,32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --quantizer turboquant --sample-interval-ms 25 --log-output review/30129-task28-ivf-a2-vacuum-scale/artifacts/ivf_vacuum_scale_1m_n8_32_64.log`

## Artifacts

- `artifacts/ivf_vacuum_scale_1m_n8_32_64.log`
- `artifacts/manifest.md`
