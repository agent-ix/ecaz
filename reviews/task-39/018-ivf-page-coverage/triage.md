# IVF Page Mutation Triage

Harness:

`make mutants MUTANTS_MODULE=src/am/ec_ivf/page.rs MUTANTS_OUTPUT_DIR=... MUTANTS_JOBS=2`

The mutation lane targets `src/am/ec_ivf/page.rs` through the
`ecaz-careful-hardening` package path so pgrx-free codec tests can exercise the
IVF page layout without loading PostgreSQL.

## Initial Run

Run: `artifacts/mutants/page.rs.mutants/mutants.out/*`

Result: 221 mutants tested, 41 missed, 143 caught, 37 unviable.

Survivor classes:

- layout constants and the posting deleted flag were not pinned directly;
- posting tuple flag/count rejection did not cover invalid flag bits or exact
  heap-TID capacity;
- centroid/list-directory fit helpers lacked negative capacity cases;
- storage format and rerank enum decode guards were only partially exercised;
- small PG-adjacent helpers were cfg-gated away from the careful harness.

## Rerun

Run: `artifacts/mutants-rerun/page.rs.mutants/mutants.out/*`

Result: 221 mutants tested, 1 missed, 182 caught, 38 unviable.

The new tests killed the real survivors. The single remaining missed mutant was
equivalent: replacing `1 << 0` with `1 >> 0` still evaluates to `1`. The flag
was rewritten as the literal `0b0000_0001` to remove that equivalent mutant
from generation.

## Final

Run: `artifacts/mutants-final/page.rs.mutants/mutants.out/*`

Result: 220 mutants tested, 0 missed, 182 caught, 38 unviable.

No missed or timeout mutants remain in the final run.
