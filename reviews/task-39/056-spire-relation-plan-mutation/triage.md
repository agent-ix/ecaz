# Triage: relation_plan.rs mutation analysis

Result: **13 mutations enumerated → 11 KILLED + 2 equivalent
(reachability-restricted), 0 non-equivalent survivors. Full
per-mutation verification under isolated CARGO_TARGET_DIR.**

## Methodology

**Full per-mutation verification** via the existing
`/tmp/run_spire_mutations_v2.py` harness with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` for build isolation (per
reviewer direction issued across 050/053/054/055). Build cycles run
in 3-10 s per mutation under the isolated target-dir vs 5-10 min
under the shared 305 GB main target/.

## Per-mutation verdicts (13 total)

11 mutations KILLED by apply/test/revert. 2 mutations MISSED — both
on line 42:28 (`relation_name.len() > max_identifier_bytes`):

- `relation_plan.rs:42:28: replace > with ==` — equivalent (see below)
- `relation_plan.rs:42:28: replace > with >=` — equivalent (see below)

## Equivalent mutants — reachability-restricted (2)

Under the careful crate build (no `pg17` / `pg18` / `pg_test`
features), `pg_identifier_limit_bytes()` returns the
`#[cfg(not(...))]` arm value `Ok(63)`. The constructed relation
name is

```
"ec_spire_store_{index_relid}_{local_store_id}"
```

= `14 + 1 + d1 + 1 + d2` bytes where `d1` and `d2` are the decimal
digit counts of `index_relid` and `local_store_id` (each 1-10 for
u32). Maximum length is therefore **16 + 20 = 36 bytes**, well below
the 63-byte limit. The `>` comparison at line 42 is
**unreachable** in the careful test surface; the `if` branch never
fires regardless of input.

- `> -> ==` keeps the branch unreachable (no name length equals 63
  exactly either) — equivalent to original under the careful build.
- `> -> >=` keeps the branch unreachable for any length below 63 —
  equivalent.

This is the same class as packets 050 (`encoded_len_after_validation`
capacity-hint) and 053 (vec_id constant arithmetic) — accepted by
the reviewer in 053's feedback as "third recurring equivalence
class: reachability-restricted `||→&&`".

The non-equivalent partner `> -> <` correctly KILLED (it inverts the
guard and would reject every valid name).

## Verification artifacts

- `artifacts/relation-plan-mutants-enumerated.txt` — full 13 enumeration.
- `artifacts/manual-verification.log` — 13/13 per-mutation verdicts
  (11 KILLED + 2 MISSED equivalent).
- `artifacts/spot-verify-plan-local-store-relations-empty.log` —
  legacy spot-verify (kept for chronological evidence).
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/relation_plan.rs` byte-for-byte
identical pre/post packet.

## Required follow-up

None. Cascade slice closed.
