# Task 28 IVF VACUUM Scale Harness

## Scope

This packet covers the first narrow smoke for the new `ecaz stress ivf-vacuum-scale` command added in `7bb54e7e`.

The harness creates one isolated table/index per `nlists` value, deletes half the rows, runs `VACUUM (ANALYZE)`, and reports:

- index size before delete, after delete, and after vacuum
- delete and vacuum wall time
- backend PID used for vacuum
- sampled backend `VmRSS` / `VmHWM` high-water values from `/proc/{pid}/status`
- sample count

## Smoke Result

PG18 smoke completed for synthetic 2k-row isolated IVF surfaces:

- nlists=8: rows 2000 -> 1000, vacuum 11 ms, index size stayed 188416 bytes, RSS/HWM peak 33960 kB
- nlists=32: rows 2000 -> 1000, vacuum 15 ms, index size stayed 196608 bytes, RSS/HWM peak 36520 kB

Raw output is in `artifacts/ivf_vacuum_scale_smoke.log`.

## Validation

- `cargo test -p ecaz-cli ivf_vacuum_scale`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 stress ivf-vacuum-scale --table-prefix task28_ivf_vacuum_scale_smoke --rows 2000 --nlists 8,32 --nprobe 8 --training-sample-rows 500 --dimensions 4 --quantizer turboquant --log-output review/30108-task28-ivf-vacuum-scale-harness/artifacts/ivf_vacuum_scale_smoke.log`
- `git diff --check`

## Next

Use this harness for the reviewer-requested 1M-row `nlists=8,32,64` A2 evidence packet. That larger run should cite the packet-local artifact and avoid relying on temporary logs.
