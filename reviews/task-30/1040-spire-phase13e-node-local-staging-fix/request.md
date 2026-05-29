# Task 30 Review Request: Node-Local Representative Staging Fix

## Summary

This fixes the AWS representative node-local load failure captured in packet 1039. The failure was an AWS node staging issue, not a SPIRE page-size or scan-path failure: the coordinator SSM command tried to download the 2.0 GiB representative corpus into `/tmp` and hit `No space left on device`.

Changes:

- `bootstrap-node.sh` grows the root filesystem when possible, uses `/var/tmp/ecaz-spire-aws` for node bootstrap staging, and removes the source build tree after installing `ecaz` and `ecaz.so`.
- `load.sh` uses configurable `/var/tmp/ecaz-spire-aws-load` for coordinator and remote representative corpus staging instead of `/tmp`, clears stale node load directories, and logs `df -h` before the large S3 download.
- `preflight-representative-performance.sh` fails closed unless those staging and cleanup guards are present.

## Validation

- `artifacts/bash-n.log`
- `artifacts/representative-preflight.log`
- `artifacts/git-diff-check.log`

Key result:

```text
SPIRE representative performance preflight passed
```

No new AWS run was started after this fix in this packet.
