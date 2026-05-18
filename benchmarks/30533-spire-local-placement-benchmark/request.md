# Review Request: SPIRE Local Placement Benchmark

- Measurement head: `54059950` (`Add SPIRE scan prefetch review packet`)
- Benchmark-driver commit: `bd311f3f` (`Add SPIRE local placement benchmark driver`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation, Phase 4 local multi-store placement
- Agent: coder1

## Summary

This packet records the first local placement benchmark for SPIRE Phase 4 on the
real 10k fixture. It includes three comparable lanes:

| lane | store layout |
| --- | --- |
| one-store baseline | root/control relation in `pg_default` |
| same-device two-store baseline | root/control plus auxiliary store, both in `pg_default` |
| extra-drive two-store lane | root/control in `pg_default`, auxiliary store in `ecaz_spire_e` at `/mnt/e/ecaz_pg_tblspc/spire_e` |

The packet also adds `scripts/bench_spire_local_placement_pg18.sh` so future
runs use one approved script instead of repeated ad hoc PG/CLI commands. The
script refuses extra-drive tablespace paths outside `/mnt/e`.

Raw logs and table outputs are under `artifacts/`; `artifacts/manifest.md`
records commands and key cited lines.

## Results

Build time for the two fresh multi-store indexes was effectively flat:

| lane | index build | full load |
| --- | ---: | ---: |
| two-store, both `pg_default` | 71.78 s | 101.69 s |
| two-store, `pg_default` + `/mnt/e` | 71.92 s | 102.44 s |

Latency over 100 queries:

| lane | nprobe | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| one-store `pg_default` | 8 | 70.1 ms | 66.7 ms | 98.4 ms | 129.7 ms |
| same-device two-store | 8 | 62.8 ms | 62.4 ms | 76.5 ms | 79.2 ms |
| `/mnt/e` two-store | 8 | 63.8 ms | 63.1 ms | 80.0 ms | 85.6 ms |
| one-store `pg_default` | 24 | 141.7 ms | 139.2 ms | 165.1 ms | 174.6 ms |
| same-device two-store | 24 | 140.6 ms | 138.2 ms | 156.7 ms | 166.8 ms |
| `/mnt/e` two-store | 24 | 143.5 ms | 141.1 ms | 163.8 ms | 180.5 ms |

Recall stayed unchanged for both two-store lanes:

| lane | nprobe | recall@10 | ndcg@10 |
| --- | ---: | ---: | ---: |
| same-device two-store | 8 | 0.9985 | 0.9999 |
| `/mnt/e` two-store | 8 | 0.9985 | 0.9999 |
| same-device two-store | 24 | 1.0000 | 1.0000 |
| `/mnt/e` two-store | 24 | 1.0000 | 1.0000 |

## Review Focus

1. Confirm that the packet is sufficient local evidence for the Phase 4 local
   placement benchmark checkpoint.
2. Check that the same-device two-store baseline is represented correctly:
   both stores use `pg_default`, proving repeated tablespace selection is
   supported.
3. Check that the `/mnt/e` lane should be treated as local extra-drive evidence
   only, not as a production multi-NVMe claim.
4. Review the benchmark script for repeatability and for the guard that avoids
   accidental `/mnt/c` or other unapproved tablespace paths.

## Validation

- `bash -n scripts/bench_spire_local_placement_pg18.sh`
- `scripts/bench_spire_local_placement_pg18.sh --help`
- `bash scripts/bench_spire_local_placement_pg18.sh --skip-load --skip-latency`
- `git diff --check`

The benchmark commands ran against local PG18. PG17 was not run because this is
a PG18 scratch measurement packet and not a PG17-facing code change.

## Notes

The `/mnt/e` drive is visible to this Linux environment through the host mount
stack. These numbers are useful as local placement and regression evidence, but
final performance claims still need true production/cloud hardware.
