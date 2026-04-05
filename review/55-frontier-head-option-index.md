# Review Request: Frontier Head Option Index

Scope:
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Replaced the scan frontier head representation from a `u8` sentinel (`u8::MAX`) to `Option<usize>`.
- Kept the current frontier contents and bootstrap scan behavior unchanged; this slice only changes how the current best candidate index is represented and consumed.
- Updated the existing debug helpers and regression tests to model “no frontier head” as `None`.

Review focus:
- Whether `Option<usize>` is the right internal head representation before the frontier starts growing beyond the current seeded pair
- Whether any lifecycle edge still assumes the old sentinel-based head state
- Whether the current debug surface is still adequate while frontier contents remain intentionally bounded

Questions to answer:
- Is removing the `u8` sentinel now the right precursor to wider frontier growth?
- Are there any remaining places where head state should stop exposing slot-oriented assumptions before real traversal expansion begins?
- Is it acceptable to keep the debug frontier snapshot two-slot-shaped for now, as long as the real head state is no longer artificially width-limited?
