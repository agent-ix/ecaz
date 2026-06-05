# Task 65b Review Request: Worker-One Scaffold

## Scope

This packet covers Task 65b Slice D: worker-count scaffolding for DiskANN graph
build without enabling true multi-worker proposal fanout yet.

The implementation now:

- sets `ec_diskann.amcanbuildparallel = true`
- wires the common parallel scan callbacks used by peer AMs
- reads graph-build worker count from PostgreSQL `IndexInfo::ii_ParallelWorkers`
  instead of a DiskANN-specific worker reloption
- keeps `ii_ParallelWorkers = 0` as the serial fallback
- supports `ii_ParallelWorkers = 1` as a rayon thread-pool boundary around the
  existing serial Vamana graph build
- rejects `ii_ParallelWorkers > 1` until Slice E implements proposal fanout
- keeps `parallel_build_batch_size` and `parallel_build_flush_rate` as
  algorithmic epoch/cache reloptions, but rejects non-default values until those
  behaviors are implemented

This packet also responds to the Slice C reviewer request by adding ADR-075 and
updating packet 003 with the concurrency-test surface, reducer floor, and Gate
#6 disposition for the rayon stepping stone.

## Code Checkpoints

- `a816c2ccf` — align DiskANN worker scaffold with PG worker controls and add
  ADR-075
- `e979a7d80` — update Slice C locking design packet after reviewer feedback
- `156c9af2b` — add explicit worker=0 fallback parity test

Earlier commit `9f4f19022` introduced the first scaffold. The later commits
address reviewer blockers by removing the custom worker-count reloption,
matching peer AM callback wiring, and adding the fallback parity test.

## Result Summary

Validation passed:

| check | result |
| --- | --- |
| `cargo fmt --check` | passed with existing stable-rustfmt warnings |
| `cargo check -p ecaz --lib --no-default-features --features pg18` | passed |
| `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann -- --test-threads=1` | passed, `190 passed` |

The single-threaded test form is intentional. A parallel test run can trip an
existing pgrx GUC guard (`postgres FFI may not be called from multiple threads`)
in `scan_profile_notice_guc_defaults_to_off`; the single-threaded run avoids
that unrelated unit-test harness issue.

## Feedback Response

The reviewer block on the initial Slice D state called out:

- missing `request.md` and `manifest.md`
- missing worker=0 fallback parity evidence
- missing peer-AM parallel callback wiring
- custom `parallel_build_workers` reloption diverging from HNSW/IVF
- rayon coordinator choice lacking an ADR / Gate #6 disposition
- silently ignored batch/flush reloptions

This packet addresses those points:

- adds this packet and manifest
- adds `task65b_worker_zero_config_matches_plain_serial_output`
- wires `amcanbuildparallel` and common parallel callbacks
- removes the custom worker-count reloption
- adds ADR-075 and updates packet 003
- rejects unsupported non-default batch/flush settings

The remaining limitation is that this is still a scaffold: `ii_ParallelWorkers =
1` runs the existing serial graph builder inside a one-thread rayon pool. Slice
E must add actual proposal fanout, stale-read accounting, reducer timing, and
the deterministic interleaving tests named in packet 003.

## Evidence

Packet-local artifacts:

- `artifacts/manifest.md`
- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-check-pg18-lib.log`
- `artifacts/cargo-test-ec-diskann-single-thread-escalated.log`

Two non-escalated test attempts are retained as artifacts because they document
the sandbox failure mode:

- `cargo-test-ec-diskann.log`
- `cargo-test-ec-diskann-single-thread.log`

Both failed at pgrx extension install with `Operation not permitted` copying to
the Homebrew PostgreSQL extension directory. The escalated single-threaded log
is the passing test evidence.

## Review Focus

- Whether using PostgreSQL `ii_ParallelWorkers` plus ADR-075 is enough to keep
  the rayon graph-core stepping stone acceptable for Slice D.
- Whether worker=0 and worker=1 parity tests cover the scaffold boundary.
- Whether rejecting `ii_ParallelWorkers > 1` and non-default batch/flush values
  is the right temporary behavior before Slice E/F.
