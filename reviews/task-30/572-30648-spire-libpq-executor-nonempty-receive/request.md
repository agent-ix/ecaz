# 30648 — SPIRE Libpq Executor Nonempty Receive

## Summary

This packet broadens the 30647 loopback executor validation from an empty
`top_k = 0` request to a nonempty candidate receive.

Code commit:

- `576a4c106f8fb7b6de2b53dbc460027b3256fb7c` — Validate SPIRE libpq executor nonempty receive

## What Changed

- Extended `test_ec_spire_remote_search_libpq_executor_loopback_empty` to also
  execute a `top_k = 1` remote search through
  `ec_spire_remote_search_libpq_executor_candidates(...)`.
- The fixture now asserts:
  - one candidate is returned,
  - the candidate node is normalized to descriptor node `2`,
  - the served epoch matches the requested coordinator epoch,
  - the row locator is nonempty.

## Validation

Focused PG18 validation passed:

```text
cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty
```

Key result:

```text
test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1463 filtered out
```

Also passed:

```text
git diff --check
```

## Review Notes

This is test-only follow-up coverage for the executor send path. It confirms the
candidate receive/validation path handles a real returned row before later
slices wire those remote candidates into coordinator heap resolution.
