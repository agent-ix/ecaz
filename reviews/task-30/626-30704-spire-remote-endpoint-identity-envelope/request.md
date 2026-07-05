# Review Request: SPIRE Remote Endpoint Identity Envelope

Status: open
Owner: coder1
Head SHA: `c134e4b859c35225edb280ba1558476f48afbad3`

## Summary

This Phase 11.3 / Stage B slice promotes the endpoint serving identity into
the `ec_spire_remote_search` candidate row envelope and closes the reviewer
P2 documentation gate from packet 30703 before further candidate-row work.

Key changes:

- Expands `ec_spire_remote_search` from the original 9 candidate columns to an
  18-column row envelope:
  `served_epoch`, `node_id`, `pid`, `object_version`, `row_index`,
  `assignment_flags`, `vec_id`, `row_locator`, `score`,
  `protocol_version`, `extension_version`, `opclass_identity`,
  `storage_format`, `assignment_payload_format`, `quantizer_profile`,
  `scoring_profile`, `profile_fingerprint`, and `endpoint_status`.
- Updates the libpq result contract and endpoint contract response row to the
  18-column shape.
- Adds libpq row decoding checks for endpoint protocol, extension version, and
  nonempty identity fields while keeping heap-candidate decoding on its older
  internal shape for the later remote-heap stage.
- Documents the v1 fingerprint input order, NUL-separated FNV-1a encoding,
  active-epoch semantics, and future training-stat extension rule in
  `plan/design/spire-remote-node-model.md`.
- Records the conservative Stage B progress in the Phase 11 task file.

## Deliberate Limits

- Production coordinator merge still needs a follow-up gate that rejects
  non-ready endpoint rows before merge. This slice exposes `endpoint_status`
  but does not yet enforce it in the coordinator merge path.
- Remote heap candidate rows are not widened in this slice; Phase 11.5 owns
  origin-node heap resolution and final row delivery.
- The fingerprint remains the v1 serving-profile fingerprint documented in
  the remote node model. Future persisted training-stat metadata must bump the
  protocol version if it changes digest semantics.

## Validation

- `cargo fmt`
  - passed; rustfmt still prints existing stable-toolchain warnings for
    unstable import-grouping settings
- `cargo test remote_search_sql_scores_selected_leaf_pids --lib`
  - passed: 1 passed, 0 failed
- `cargo test remote_search_receive_contract --lib`
  - passed: 1 passed, 0 failed
- `cargo test remote_search_libpq_req --lib`
  - passed: 2 passed, 0 failed
- `git diff --check`
  - passed

## Review Focus

- Is the 18-column endpoint envelope the right v1 shape before production
  coordinator enforcement?
- Is it correct to validate protocol/version/nonempty identity fields in libpq
  decode while leaving `endpoint_status` enforcement for the next coordinator
  gate?
- Does the remote-node-model fingerprint documentation satisfy packet 30703 P2
  before this row shape becomes the stable wire contract?
