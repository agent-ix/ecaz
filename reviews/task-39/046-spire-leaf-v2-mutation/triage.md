# Triage: leaf_v2.rs mutation campaign

For each of the 14 mutations enumerated by
`cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/leaf_v2.rs --list`,
the table below names the existing killing test from packets 029 and
044 and the verification status from `artifacts/manual-verification.log`.

| Mutant | Operator swap | Killing test (existing) | Verdict |
| --- | --- | --- | --- |
| 18:9  | `validate -> Ok(())` | every `miri_leaf_v2_validate_rejects_*` test calls `column_segments().err()`, which would Ok-out and panic the `.expect_err` | KILLED (M1) |
| 20:36 | `!=` -> `==` (segment_count guard) | `miri_leaf_v2_validate_rejects_segment_count_mismatch` | KILLED (M2) |
| 31:35 | `!=` -> `==` (segment_no guard) | `miri_leaf_v2_validate_rejects_segment_number_mismatch` | KILLED (M3) |
| 37:33 | `!=` -> `==` (row_base guard) | `miri_leaf_v2_validate_rejects_row_base_mismatch` | KILLED (M4) |
| 46:40 | `==` -> `!=` (final-segment select) | `miri_leaf_v2_validate_rejects_final_segment_with_non_invalid_locator` | KILLED (M5) |
| 46:36 | `+` -> `-` (final-segment select) | every validate test panics on `0u32 - 1` underflow | KILLED (M6) |
| 46:36 | `+` -> `*` (final-segment select) | `miri_leaf_v2_validate_rejects_final_segment` / `non_final` / `meta_assignment` all flip branch | KILLED (M7) |
| 47:49 | `!=` -> `==` (final-segment locator) | `miri_leaf_v2_validate_rejects_final_segment_with_non_invalid_locator` | KILLED (M8) |
| 52:52 | `==` -> `!=` (non-final locator) | `miri_leaf_v2_validate_rejects_non_final_segment_missing_locator` | KILLED (M9) |
| 56:46 | `!=` -> `==` (meta assignment_count) | `miri_leaf_v2_validate_rejects_meta_assignment_count_mismatch` | KILLED (M10) |
| 68:9  | `column_segments -> Ok(empty())` | every `miri_leaf_v2_validate_rejects_*` test (validate short-circuited) | KILLED (M11) |
| 68:9  | `column_segments -> Ok(once(default))` | same as above | KILLED (M12) |
| 76:9  | `assignment_rows -> Ok(vec![])` | `miri_leaf_v2_assignment_rows_round_trips_segments_back_to_rows` (len assert) | KILLED (M13) |
| 76:9  | `assignment_rows -> Ok(vec![Default])` | same — len 1 ≠ expected 2 | KILLED (M14) |

**Net result: 14 caught, 0 missed, 0 timed-out.** No new tests were
required; the existing tests from packets 029 and 044 already
distinguish every operator swap and body replacement.

## Why this packet runs the mutations manually

The reviewer's prescribed invocation
(`cargo mutants --package ecaz-careful-hardening --file hardening/careful/src/../../../src/am/ec_spire/storage/leaf_v2.rs -j 4`)
returns `Found 0 mutants to test  WARN No mutants found under the active filters` —
the production source mounts SPIRE storage children via
`include!` (`src/am/ec_spire/storage.rs::include!("storage/leaf_v2.rs")`),
and the careful crate's `pub mod storage { include!(...) }` block does
the same. `cargo mutants` discovers mutations by walking `mod`/`pub
mod` declarations; content reached only through `include!` is invisible
to its source map. The reviewer's note that "packet 021 brought the
SPIRE storage codecs in via cfg-gating; the same harness runs cargo
mutants" assumes the codecs are visible as ordinary submodules; they
are not.

Two paths forward for the rest of the SPIRE storage cascade
(leaf_v2_parts, header, helpers, assignment, local_store,
local_store_set, vec_id, routing_delta, top_graph, relation_plan,
leaf_v1, page) were considered:

1. Continue the per-file manual verification pattern used in this
   packet — enumerate via `--Zmutate-file`, apply each mutation
   transiently, run the careful suite with a focused filter, record
   pass/fail, revert.
2. Restructure `src/am/ec_spire/storage.rs` (and/or the careful
   crate's `pub mod storage` block) from `include!` to `mod`
   declarations so cargo-mutants discovers each child file. This is
   the smaller of the two — only the careful crate side has to
   change — but it still requires adding `use super::*;` (or
   equivalent imports) to each child file in either the production
   tree or the careful mount, because every child currently relies
   on the parent's flat-namespace scope.

Path (1) is what this packet uses and is the path the next 12
SPIRE-storage packets should also use unless the reviewer
authorises the include!-to-mod refactor in path (2). The verification
takes ~30 seconds per file once the helper scripts are reused.

## Validation

- `artifacts/leaf-v2-mutants-enumerated.txt` — 14 mutations enumerated.
- `artifacts/initial-mutants-run.log` — confirms the reviewer's
  cargo-mutants invocation finds 0 mutants under `include!`.
- `artifacts/manual-verification.log` — per-mutation
  verification (apply + careful test + revert) with KILLED/MISSED
  verdicts.
- `artifacts/post-verification-tests.log` — full careful suite
  green at 529 passing after every mutation was reverted.

No production code change. Triage map and per-mutation verdict are
recorded so a reviewer can re-run any single mutation by editing the
source and running the corresponding `miri_leaf_v2_*` test.
