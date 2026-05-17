# Review Request: Benchmark Reporting Standard

## Scope

This docs/spec checkpoint adds a repo-wide benchmark reporting standard for all
access methods, quantizers, storage formats, and option sets.

Code/evidence commit: `4f536bc3`

## Changed Surface

- Adds `NFR-015 Benchmark Reporting Standard` as the normative spec anchor.
- Adds `docs/benchmark-reporting-standard.md` as the public reporting standard.
- Updates `StR-006`, `US-015`, `US-017`, `FR-038`, `spec/spec.md`, and
  `spec/tests.md` to trace the new reporting standard.
- Updates README, usage, benchmarks, benchmark index, and the legacy
  `BENCHMARKS.md` template to point at the new standard.
- Adds current quantizer/storage candidate packet links for IVF and SPIRE,
  including RaBitQ as the first SPIRE remote-serving storage profile while
  preserving the current IVF latency caveat.

## Review Focus

1. Confirm `NFR-015` is the right spec home for the standard rather than
   overloading `NFR-007`.
2. Check that the required fields cover current and future comparisons across
   `ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire`, trained quantizers, and
   future formats.
3. Check that RaBitQ/PQ-FastScan wording is evidence-grounded and does not
   promote local/review-packet evidence to product claims.
4. Check that docs links and traceability rows point to the right surfaces.

## Validation

No runtime tests were run. This checkpoint changes docs/spec only.

- `git diff --check`
- local markdown-link check over the changed docs/spec files

See `artifacts/manifest.md` for validation metadata.
