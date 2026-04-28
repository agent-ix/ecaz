# Task 28 IVF A2 1M VACUUM Scale Evidence

## Scope

This packet records the reviewer-requested A2 1M-row IVF VACUUM scale run using the `ecaz stress ivf-vacuum-scale` harness.

The fixture uses isolated one-index-per-table surfaces for `nlists=8,32,64`, deletes half of each table, and samples backend memory only during `VACUUM (ANALYZE)`.

## Result

The 1M synthetic half-delete run completed:

- nlists=8: vacuum 2305 ms, index 89055232 bytes before/after, RSS peak 368328 kB, HWM peak 430580 kB, 87 samples
- nlists=32: vacuum 2034 ms, index 89055232 bytes before/after, RSS peak 373188 kB, HWM peak 435688 kB, 77 samples
- nlists=64: vacuum 2059 ms, index 89063424 bytes before/after, RSS peak 373724 kB, HWM peak 436228 kB, 77 samples

The run does not claim physical index shrinkage. The stable index byte counts are expected for the current v1 tombstone/reuse design.

Raw output is in `artifacts/ivf_vacuum_1m_n8_32_64.log`.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_vacuum_1m --rows 1000000 --nlists 8,32,64 --nprobe 8 --training-sample-rows 10000 --dimensions 4 --quantizer turboquant --log-output review/30109-task28-ivf-a2-1m-vacuum-scale/artifacts/ivf_vacuum_1m_n8_32_64.log`

Earlier harness validation for the same head lineage:

- `cargo test -p ecaz-cli ivf_vacuum_scale`
- `git diff --check`

## Interpretation

This closes the immediate A2 evidence gap for a 1M-row local synthetic fixture: the streaming VACUUM path avoids an obvious nlists=8 memory cliff at this scale, with measured HWM staying within a narrow 430580-436228 kB band across the three list counts.

The next vacuum-related work remains A3 replacement convergence for the n32/n64 churn cases and future physical compaction if the project wants to claim space reclamation rather than reuse.
