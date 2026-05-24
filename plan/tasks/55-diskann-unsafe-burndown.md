# Task 55: DiskANN Unsafe Burndown

Status: **proposed** — first per-AM rotation after Phase 1 common
wrappers land. Supersedes the per-AM scope of Task 50 by applying the
same recipe to `src/am/ec_diskann/`.

## Why

`src/am/ec_diskann/` carries **65** `unsafe { ... }` blocks across
seven files. It is the smallest remaining AM target and has no
active optimization work (unlike SPIRE and IVF, which are blocked on
their own optimization branches per memory
`feedback_main_priority_in_conflicts`).

DiskANN is therefore the safest, fastest validation lane for the
Phase-1 wrappers landed by Tasks 52 (P8) / 53 (P6) / 54 (P3). If the
wrappers work for HNSW + DiskANN, they're proven cross-AM patterns;
if a wrapper needs extension for DiskANN's variant, that extension
lands during this task before SPIRE/IVF rotations consume them.

## Non-Goals

- Do not touch `src/am/ec_hnsw/**`, `src/am/ec_ivf/**`, or
  `src/am/ec_spire/**`. SPIRE/IVF are gated; HNSW is closed.
- Do not extend Phase-1 wrappers beyond what DiskANN needs. Wrapper
  extension is allowed but must be commit-separated from consumer
  migration and must land in the owning Phase-1 module (`dsm.rs`,
  `datum.rs`, `storage/*.rs`).
- Do not run a SIMD micro-optimization pass. DiskANN scoring math is
  unchanged.

## Scope

Audit and structurally reduce `unsafe { ... }` blocks across all of
`src/am/ec_diskann/`. Current distribution:

| File | Count |
| --- | ---: |
| `routine.rs` | 27 |
| `ambuild.rs` | 19 |
| `insert.rs` | 8 |
| `scan_state.rs` | (counted) |
| `scan_state` other helpers | (counted) |
| **DiskANN subsystem total** | **65** |

(Refresh exact per-file counts at task open via direct grep.)

## Techniques

Apply the Task 50 recipe with Phase-1 wrappers as consumers:

1. **Safe-fn lifts** — `unsafe fn` → `fn` once dependencies are
   already safe.
2. **Narrow block scoping** — split wide `unsafe { ... }` into the
   smallest expression that requires it.
3. **Consume Phase-1 P6 datum wrappers** in `routine.rs` and
   `insert.rs` for `Datum → EcVector` extraction.
4. **Consume Phase-1 P3 page/WAL wrappers** in `ambuild.rs` and the
   build-side page-mutation paths.
5. **Consume Phase-1 P8 wrappers** if DiskANN has any parallel-build
   path (verify at task open; otherwise skip).

## Migration Targets

| File | Now | Target | Δ |
| --- | ---: | ---: | --- |
| `routine.rs` | 27 | ≤ 16 | -41% (P6 unlock primary) |
| `ambuild.rs` | 19 | ≤ 11 | -42% (P3 unlock primary) |
| `insert.rs` | 8 | ≤ 5 | -38% |
| `scan_state.rs` and other helpers | residual | residual | safe-fn lifts |
| **DiskANN subsystem total** | **65** | **≤ 40** | **-38% or better** |

Per Task 50 §Exit Criteria, ≥ -30% per-module is the floor unless a
structural-ceiling rationale is documented. DiskANN should comfortably
exceed -30% given Phase-1 wrappers exist when this task opens.

## Slice and Packet Rules

Same as Tasks 50 / 52-54. Specifically:

- Each packet must report `unsafe { ... }` block count before / after
  for every touched file, plus `src/` total.
- Bench evidence is **per task**, not per slice, per memory
  `feedback_coder_push_smoke_checks` (smoke checks between slices,
  bench window once at close).
- If a slice has to extend a Phase-1 wrapper, the extension lands
  separately in the owning Phase-1 module, with its own bench gate.

## Performance Gate

DiskANN's low-L latency curves are the relevant bench lane per Task
50's §Performance Gate template. Required evidence at task close:

- New benchmark packet `benchmarks/task-55-m5-diskann-baseline/`
  modeled on `benchmarks/task-50-m5-hnsw-baseline/`. Suite config
  drives DiskANN's standard sweeps on the M5 host.
- Recall + latency + storage must not regress beyond noise.

Pre-state baseline: there is none — DiskANN does not have a
post-burndown reference yet. This task establishes one.

## Validation

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- focused `cargo pgrx test pg18 ec_diskann::*` when behavior could
  plausibly drift
- direct unsafe-block count per touched file
- `src/` total snapshot

## Exit Criteria

Task closes when:

- Each touched file's block count has dropped by at least 30%, or the
  request explains why a lower reduction is the structural ceiling
  (same gate as Task 50).
- DiskANN subsystem total ≤ 40.
- DiskANN bench packet at `benchmarks/task-55-m5-diskann-baseline/`
  runs cleanly with 8/8 steps succeeded.
- A closing summary packet records:
  - final per-file distribution;
  - explicit list of Phase-1 wrappers consumed (P8 / P6 / P3);
  - any Phase-1 wrapper extensions landed (and the owning Task
    52/53/54 commit SHAs);
  - the `src/` total block count change;
  - structural-ceiling rationale for any sub-30% file.

## Coordination

- **Gate**: opens only after Tasks 52 (P8), 53 (P6), and 54 (P3)
  close. Phase-1 wrappers must exist.
- Runs in parallel with the deferred Tasks 56 (SPIRE) / 57 (IVF)
  only if those open in their respective deferred windows; otherwise
  serial.
- Reviewer scope-lock: DiskANN-only on this branch
  (`task-55-diskann-burndown`).
- No overlap with Task 51 (IVF RaBitQ optimization) — different AM.

## Cross-References

- Inherits Task 50 §Performance Gate template.
- Consumes wrappers from Tasks 52, 53, 54.
- Establishes the M5 DiskANN baseline that future Task 55 / Task 33
  packets compare against.
