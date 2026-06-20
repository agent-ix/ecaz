# Task 111h / Packet 022 Review Request: Residual RaBitQ Rerank Sidecars

## Summary

This packet requests review for a corrective Task 111h code checkpoint:

- `7112caeae6ac5bfc659253e823d13a3f31f64b2e` `task111h: use residual rabitq rerank sidecars`

The prior RaBitQ4/RaBitQ8 index-side rerank sidecar path encoded whole source
vectors and scored them with a zero centroid correction. That bypassed the
residual machinery added by Task 115 and makes the existing compressed-format
recall benchmark packets suspect for RaBitQ.

This checkpoint changes the index-side RaBitQ sidecar path to encode
`source - assigned_centroid` at build/insert time, and changes scan-time
rerank scoring to add back the selected-list centroid inner product:

```text
final distance = -(residual_estimate + centroid_ip)
```

Source-diagnostic conversion remains a non-residual query-time baseline. If an
index has no rerank sidecar group for a tuple, scan fallback still uses that
source-diagnostic scorer rather than pretending a residual sidecar exists.

## Format Version

The persisted sidecar layout size did not change, but RaBitQ sidecar payload
semantics did. The checkpoint therefore bumps IVF from v5 to v6, updates the
current fixture, and marks v5 as unreadable/unwritable in the upgrade matrix so
old non-residual RaBitQ sidecar bytes cannot be interpreted as residual bytes.

## Validation

Packet-local logs:

- `artifacts/cargo-test-rabitq-residual-sidecar.log`
- `artifacts/cargo-test-payload-codecs.log`
- `artifacts/cargo-test-ivf-metadata.log`
- `artifacts/cargo-test-upgrade-matrix.log`
- `artifacts/cargo-test-rerank-group-lookup.log`

Result summary:

```text
index_side_rabitq_payloads_require_centroid_and_apply_correction: 1 passed
payload_codecs: 2 passed
ivf_metadata_: 5 passed
upgrade_matrix: 2 passed
rerank_group_payload_lookup: 2 passed
```

See `artifacts/manifest.md` for commands and exact result lines.

## Review Ask

Please review whether residual RaBitQ sidecar encode/score is wired correctly
through build, insert, scan, and the group lookup path.

Please also review whether the v6 format bump, fixture update, docs, and
upgrade matrix handling are sufficient for the semantic incompatibility with
v5 sidecar payloads.

## Non-Claims

This packet does not claim new recall, latency, or storage results. The older
Task 111h RaBitQ4/RaBitQ8 benchmark numbers were gathered before this fix and
should not be treated as final evidence for the compressed rerank design.

This slice also does not answer whether TurboQuant should become residual or
centroid-relative. That remains a separate investigation.
