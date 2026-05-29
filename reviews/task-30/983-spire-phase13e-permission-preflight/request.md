# Review Request: SPIRE Phase 13e Permission Preflight

Requester: coder1
Date: 2026-05-25
Head SHA: `b677cdc3fb24839259d8c82649a0bbbaccdf5352`
Review focus: verify AWS permission gaps now fail before SPIRE AWS provisioning instead of surfacing mid-run.

## Summary

This slice adds `scripts/spire-aws/preflight-permissions.sh` and wires it into
`make -C infra/spire-aws provision`.

The preflight is read-only. It checks:

- current AWS identity via STS;
- SPIRE artifact buckets by prefix;
- `s3:ListBucketVersions` on matching buckets, which is required for versioned cleanup and Terraform `force_destroy`;
- Secrets Manager list access for matching SPIRE secrets.

With the current `ecaz-operator` identity it fails before provisioning because two old SPIRE buckets exist and the identity lacks `s3:ListBucketVersions` on both. Secrets Manager list access is OK.

No AWS resources were provisioned, modified, or deleted for this packet.

## Validation

- `bash -n scripts/spire-aws/preflight-permissions.sh scripts/spire-aws/cleanup-residue.sh scripts/spire-aws/preflight-state.sh scripts/spire-aws/preflight-operator.sh` passed.
- `make -C infra/spire-aws preflight-permissions` fails with the expected `s3:ListBucketVersions` blocker.
- `git diff --check` passed.

See `artifacts/manifest.md` for packet-local logs.

## Remaining Blocker

The corrected Graviton AWS correctness run should not start until this preflight passes, or until a reviewer accepts a packet-local exception for the old buckets and stale state.
