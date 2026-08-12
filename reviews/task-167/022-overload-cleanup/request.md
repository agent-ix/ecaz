# Task 167 overload-cleanup review request

Please independently review code checkpoint `fb6a9cd55`.

The broad PG18 `test_distann_remote_endpoint_acl_class` exposed an ambiguity
in the physical `ec_distann_expand_nodes` SQL wrappers: two overloads used
defaults that made a five-argument call non-unique. This checkpoint removes
only the conflicting defaults. Coordinator/owner callers already pass the
explicit full argument lists; the physical endpoint ACL test now completes
successfully.

Validation:

- `cargo pgrx test pg18 test_distann_remote_endpoint_acl_class --no-default-features --features pg18,pg_test`
- Result: `1 passed; 0 failed; 2577 filtered out`; the full endpoint ACL class,
  including physical DML endpoints, passes.

Evidence is in [`artifacts/overload-cleanup.log`](artifacts/overload-cleanup.log)
and [`artifacts/manifest.md`](artifacts/manifest.md). This packet remains
review-open pending an outside reviewer verdict.
