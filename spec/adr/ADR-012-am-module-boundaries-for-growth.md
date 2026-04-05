# ADR-012: AM Module Boundaries For Growth

## Status

Accepted

## Context

The access-method implementation has moved past bootstrap scaffolding and now has multiple
independent growth areas:

- build path
- scan execution and future graph traversal
- live insert, which will later become graph-aware
- vacuum/maintenance behavior

The scan path in particular is expected to grow materially through candidate/result state, visited
tracking, entry-point descent, graph page reads, and final ordered result emission. Letting that
growth continue in `src/am/mod.rs` would force repeated structural refactors while the traversal
implementation is still in motion.

## Decision

Use explicit AM module boundaries oriented around expected future growth:

- `src/am/build.rs` owns build-time tuple decoding, graph construction, and staged page writes.
- `src/am/scan.rs` owns scan descriptor lifecycle, scan-local state, bootstrap linear scan, and
  future ordered traversal machinery.
- `src/am/mod.rs` remains the shared AM coordination layer plus the current live insert path and
  low-level shared helpers.
- `src/am/routine.rs`, `src/am/cost.rs`, `src/am/options.rs`, and `src/am/vacuum.rs` remain thin
  surface modules.

Future growth should follow these rules:

- ordered search, candidate heaps, visited sets, graph descent, and score/result bookkeeping land
  under `scan`
- `mod.rs` should not regain responsibility for large scan-execution features
- if live insert grows materially for graph maintenance, that path should move into its own module
  rather than expanding `mod.rs`
- if `scan.rs` itself becomes too large, split it by execution concern rather than by arbitrary
  helper count:
  - `scan/state`
  - `scan/linear`
  - `scan/graph`
  - `scan/debug`

## Consequences

Positive:

- traversal work now has a clear home that does not compete with build or AM shell code
- future graph-scan slices can accumulate without another immediate top-level AM reshuffle
- `mod.rs` becomes a coordination boundary instead of a catch-all implementation file

Negative:

- a few shared low-level helpers still live in `mod.rs`, so the boundary is cleaner than before
  but not yet final
- graph-aware insert work will likely require another extraction later, this time from `mod.rs`
  into a dedicated insert module

## Notes

This ADR is intentionally about growth direction, not final file count. The goal is to prevent the
expected traversal and graph-maintenance phases from re-centralizing implementation in `mod.rs`.
