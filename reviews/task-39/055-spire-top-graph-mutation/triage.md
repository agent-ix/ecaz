# Triage: top_graph.rs mutation analysis

Result: **62 mutations enumerated → 62 KILLED, 0 MISSED, 0 timeouts
(full per-mutation verification under isolated CARGO_TARGET_DIR,
plus three new boundary-killing tests).**

## Methodology

**Full per-mutation verification** via
`/tmp/run_spire_mutations_v2.py` with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation, per
reviewer direction.

## Per-mutation verdicts (62 total)

59 mutations KILLED by the existing top_graph test surface
(round-trip + 12 validate-rejects-* tests). 3 mutations initially
MISSED, all on `<=` / `>=` boundary checks where no existing test
probed the exact boundary; killed after adding 3 new tests.

## New killing tests

Added to `src/am/ec_spire/storage/tests/top_graph.rs`:

| Test | Kills | Mechanism |
| --- | --- | --- |
| `top_graph_partition_object_decode_rejects_prefix_only_tail_at_boundary` | `top_graph.rs:91:23 < -> <=` (decode prefix-len guard) | Truncates encoded bytes to `header_bytes + 28` (TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES). Asserts error contains `"node 0 body too short"`. Mutant errors at line 91 with `"object body too short"`, missing the `"node 0"` prefix. |
| `top_graph_partition_object_accepts_alpha_exactly_one` | `top_graph.rs:240:50 < -> <=` (validate `alpha < 1.0` guard) | Constructs valid top_graph with `alpha = 1.0`. Original accepts (alpha == 1.0 is valid); mutant rejects (`1.0 <= 1.0` true). |
| `top_graph_partition_object_accepts_neighbor_count_equal_to_degree` | `top_graph.rs:278:37 > -> >=` (validate neighbor-count guard) | Constructs top_graph with `graph_degree = 2` and a node with `neighbors.len() == 2`. Original accepts (`2 > 2` false); mutant rejects (`2 >= 2` true). |

Mutant re-verification after the tests were added:

| Mutant | Original | Mutant |
| --- | --- | --- |
| `< -> <=` line 91 | 553 passed | **1 failed** |
| `< -> <=` line 240 (alpha) | 553 passed | **1 failed** |
| `> -> >=` line 278 (neighbors) | 553 passed | **1 failed** |

## Verification artifacts

- `artifacts/top-graph-mutants-enumerated.txt` — full 62 enumeration.
- `artifacts/manual-verification.log` — 62/62 per-mutation verdicts.
- `artifacts/spot-verify-encode-body-replacement.log` — legacy
  spot-verify kept for chronology.
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/top_graph.rs` byte-for-byte
identical pre/post packet (only the test file gained 3 new tests).
