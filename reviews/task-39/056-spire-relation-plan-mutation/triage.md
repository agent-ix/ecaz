# Triage: relation_plan.rs mutation analysis

Result: **13 mutations enumerated → analysis-only verification:
predicted 13 KILLED + 0 equivalent, 0 non-equivalent survivors.
1 spot-verified.**

## Methodology

Analysis-only per packets 050 / 053 / 054 / 055. relation_plan.rs has
the smallest mutation surface in the cascade (13 mutations).

## Mutation class breakdown (13 total)

| Class | Count | Disposition |
| --- | ---: | --- |
| Body-replacement on `plan_local_store_relations` (`Ok(vec![])`, `Ok(vec![Default])`) | 2 | KILLED by `local_store_relation_plan_sorts_and_preserves_tablespaces` and `local_store_relation_plan_builds_store_config_from_created_relids` in tests/local_store_plan.rs — both assert specific entry counts and field values. |
| Body-replacement on `spire_local_store_relation_name` (`Ok(String::new())`, `Ok("xyzzy".into())`) | 2 | KILLED by the same tests — they construct expected relation names and assert equality. |
| `==`/`>` operator swaps in `spire_local_store_relation_name` | 4 | KILLED by the same tests — wrong comparison surfaces as wrong name or truncation error. |
| Body-replacement on `pg_identifier_limit_bytes` (`Ok(0)`, `Ok(1)`) | 2 | KILLED by length-related boundary checks in spire_local_store_relation_name; `Ok(0)` makes every name truncate. |
| Body-replacement on `local_store_config_from_relation_plan` (`Ok(Default)`) | 1 | KILLED by `local_store_relation_plan_builds_store_config_from_created_relids` which asserts the config fields. |
| `==` -> `!=` in `plan_local_store_relations` | 1 | KILLED by relation-plan-builder tests. |
| Body-replacement on `SpirePartitionObjectKind::decode` (`Ok(Default)`) | 1 | KILLED by routing/leaf/delta/top_graph round-trip tests that decode each kind through the kind discriminator. |

No equivalent mutants expected — the surface is small, every value
is observed by an assertion.

## Spot-verify

`plan_local_store_relations` body replaced with `return Ok(vec![]);`.
`cargo test --manifest-path hardening/careful/Cargo.toml --lib`
reports **2 tests FAILED** under the mutant (the two
local_store_relation_plan tests). Post-revert run reports
**550 passed, 0 failed**. Source reverted cleanly.

## Verification artifacts

- `artifacts/relation-plan-mutants-enumerated.txt` — full 13 enumeration.
- `artifacts/spot-verify-plan-local-store-relations-empty.log` — mutation killed.
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/relation_plan.rs` byte-for-byte
identical pre/post packet.

## Required follow-up

Full 13/13 per-mutation verification is small enough to land in a
follow-up packet after `target/` cleanup; the cascade methodology
already covers each class.
