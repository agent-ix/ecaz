#!/usr/bin/env bash
# Local self-check that cleanup-residue is idempotent when a security group is
# deleted by another teardown path between discovery and cleanup.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/spire-cleanup-local.XXXXXX")"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

bin_dir="$work_dir/bin"
state_dir="$work_dir/state"
mkdir -p "$bin_dir" "$state_dir"

cat > "$bin_dir/aws" <<'BASH'
#!/usr/bin/env bash
set -euo pipefail

state_dir="${SPIRE_CLEANUP_LOCAL_STATE:?state path required}"
service="${1:?service required}"
operation="${2:?operation required}"
shift 2

case "${service}:${operation}" in
  s3api:list-buckets)
    printf '\n'
    ;;
  secretsmanager:list-secrets)
    printf '\n'
    ;;
  iam:get-role|iam:get-instance-profile)
    printf 'not found\n' >&2
    exit 254
    ;;
  ec2:describe-vpcs)
    printf 'vpc-local\n'
    ;;
  ec2:describe-instances|ec2:describe-vpc-endpoints|ec2:describe-route-tables|ec2:describe-subnets)
    printf '\n'
    ;;
  ec2:describe-security-groups)
    if [[ ! -f "$state_dir/sg-listed" ]]; then
      touch "$state_dir/sg-listed"
      printf 'sg-local\n'
      exit 0
    fi
    printf "An error occurred (InvalidGroup.NotFound) when calling the DescribeSecurityGroups operation: The security group 'sg-local' does not exist\n" >&2
    exit 254
    ;;
  ec2:delete-security-group)
    printf "An error occurred (InvalidGroup.NotFound) when calling the DeleteSecurityGroup operation: The security group 'sg-local' does not exist\n" >&2
    exit 254
    ;;
  ec2:delete-vpc)
    exit 0
    ;;
  *)
    printf 'unexpected aws call: %s %s %s\n' "$service" "$operation" "$*" >&2
    exit 64
    ;;
esac
BASH

chmod +x "$bin_dir/aws"

if ! PATH="$bin_dir:$PATH" \
  SPIRE_CLEANUP_LOCAL_STATE="$state_dir" \
  scripts/spire-aws/cleanup-residue.sh --execute \
    > "$work_dir/cleanup.stdout" \
    2> "$work_dir/cleanup.stderr"; then
  cat "$work_dir/cleanup.stdout" >&2
  cat "$work_dir/cleanup.stderr" >&2
  exit 1
fi

if ! grep -q 'Security group sg-local was already deleted before rule cleanup' "$work_dir/cleanup.stdout"; then
  printf 'ERROR: cleanup did not treat missing security group as already deleted\n' >&2
  cat "$work_dir/cleanup.stdout" >&2
  cat "$work_dir/cleanup.stderr" >&2
  exit 1
fi

cat "$work_dir/cleanup.stdout"
printf 'SPIRE AWS cleanup residue local self-check passed\n'
