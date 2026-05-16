---
topic: spire-multirow-gid-feedback
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30930
stage: phase-12.4
status: open
---

# Review Request: SPIRE Multi-Row GID Feedback

## Scope

Please review commit `6f06297f34db6fb60365ae557f22d622dba1673b`
(`Document SPIRE multi-row GID limits`).

This is a focused response to reviewer feedback on packet `30928`:

- Adds a comment on the test-only `tests.ec_spire_test_set_env_var(...)`
  helper explaining why loopback trigger-dispatch fixtures need it.
- Updates ADR-069's prepared-transaction GID section to state the v1
  limitation surfaced by the multi-row fixture: multi-row INSERTs in one
  coordinator transaction are only supported when per-row routing targets
  different `node_id` values; same-node multi-row INSERTs remain unsupported
  until async/batched dispatch consolidates multiple rows into one prepared
  remote transaction per node.

No runtime behavior changed.

## Review Focus

- Confirm the helper comment is enough to document the test-only env mutation
  constraint.
- Confirm the ADR limitation matches the current GID shape
  `(index_oid, node_id, served_epoch, top_xid)` and does not overclaim P9
  async-dispatch delivery.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
