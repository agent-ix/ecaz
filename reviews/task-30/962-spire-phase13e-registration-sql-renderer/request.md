# Review Request: SPIRE Registration SQL Renderer

Task: Task 30 Phase 13e SPIRE AWS Production Gap Closure

Code commit: `7bba11ff61fdb7e2999c5b204acb7dc20f1dac9e`

## Summary

This slice adds `ecaz corpus render-spire-registrations`, an offline operator
step that converts the distributed placement plan plus per-remote identity JSON
into coordinator SQL for `ec_spire_register_remote_node_descriptor`.

The renderer is intentionally fail-closed:

- rejects empty/duplicate remote descriptors in the placement plan
- rejects mismatched remote index identity files
- requires both endpoint and tuple transport status to be `ready`
- validates nonempty even-length hex endpoint identities before writing SQL
- preserves explicit descriptor generation input and SQL literal escaping

This removes the remaining hand-edit/template step between remote index build
and coordinator descriptor registration. It does not yet execute remote loads or
publish coordinator placement-directory rows.

## Evidence

See `artifacts/manifest.md`.

- `artifacts/cargo-test-ecaz-cli-render-spire-registration.log`: focused unit
  validation for the renderer, `5 passed; 0 failed`.

## Reviewer Notes

Please focus on the generated SQL contract and validation boundaries. The next
13e.1 slice should use this renderer in a local coordinator+remote fixture so
descriptor registration and remote placement publication are proven end to end.
