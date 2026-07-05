#!/usr/bin/env bash
# Require an explicit operator confirmation before provisioning AWS resources.

set -euo pipefail

expected="${SPIRE_AWS_CONFIRM_PROVISION_EXPECTED:-yes}"
actual="${SPIRE_AWS_CONFIRM_PROVISION:-}"

if [[ "$actual" != "$expected" ]]; then
  cat >&2 <<EOF
ERROR: refusing to provision SPIRE AWS resources without explicit confirmation.

Set:

  SPIRE_AWS_CONFIRM_PROVISION=${expected}

This guard runs before Terraform provisioning so plain make pass targets cannot
accidentally start EC2 instances. Preflight and teardown targets do not require
this confirmation.
EOF
  exit 2
fi

printf 'SPIRE AWS provisioning confirmation accepted\n'
