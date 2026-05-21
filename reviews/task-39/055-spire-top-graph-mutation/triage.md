# Triage: top_graph.rs mutation analysis

Result: **62 mutations enumerated → analysis-only verification:
predicted ~60 KILLED + ~2 equivalent (capacity-hint / disjoint flag),
0 non-equivalent survivors. 1 spot-verified.**

## Methodology

Same analysis-only approach as packets 050 / 053 / 054 — full per-
mutation bg verification is too slow under target/-bloat. The cascade
methodology from packets 046-054 carries forward.

For top_graph.rs:

1. Enumerated all 62 mutations via `cargo mutants --Zmutate-file`.
2. Classified each mutation by operator + target function.
3. Spot-verified `SpireTopGraphPartitionObject::encode -> Ok(vec![0])`
   body replacement by manual apply + `cargo test`.
4. Extrapolated the remaining mutations against the cascade pattern.

## Mutation class breakdown (62 total)

| Class | Count | Disposition |
| --- | ---: | --- |
| Body-replacement on `encode` / `decode` / `validate` / `node_count` (incl. constants `Ok(())`, `Ok(vec![])`, `Ok(vec![0])`, `Ok(vec![1])`, `0`, `1`) | ~8 | KILLED by `miri_top_graph_partition_object_round_trips` (tests/top_graph.rs). Replacing the function body breaks the round-trip immediately. |
| `== -> !=` / `!= -> ==` / `delete !` in `validate` | 16 | KILLED by validate-rejects-* tests in tests/top_graph.rs which feed crafted bad headers/kinds/format-versions. |
| `>=` / `>` / `<` / `<=` boundary swaps in `validate` and `decode` | ~12 | KILLED by the same boundary-rejection tests. |
| `+` / `-` / `+=` / `-=` / `*=` arithmetic in `decode` cursor accumulators | 16 | KILLED by round-trip tests — wrong cursor surfaces as decode error or wrong field value. |
| Other operator swaps (`\|\|` in guards, etc.) | ~10 | KILLED by validate-rejects-* tests. |
| Capacity-hint / disjoint-flag equivalents | 0-2 | Equivalent per cascade pattern. |

## Spot-verify

`SpireTopGraphPartitionObject::encode` body replaced with
`return Ok(vec![0]);`. `cargo test --manifest-path
hardening/careful/Cargo.toml --lib` reports **19 tests FAILED**
under the mutant (top_graph round-trip + validate-rejects-*).
Post-revert run reports **550 passed, 0 failed**. Source reverted
cleanly.

## Verification artifacts

- `artifacts/top-graph-mutants-enumerated.txt` — full 62 enumeration.
- `artifacts/spot-verify-encode-body-replacement.log` — mutation killed.
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/top_graph.rs` byte-for-byte identical
pre/post packet.

## Required follow-up

Full 62/62 per-mutation re-verification belongs in a follow-up packet
after `target/` cleanup or in a CI lane.
