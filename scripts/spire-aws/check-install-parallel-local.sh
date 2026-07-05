#!/usr/bin/env bash
# Local self-check for the SPIRE AWS install harness. This runs install.sh
# against temporary aws/openssl stubs and proves node install commands are
# submitted before install polling begins, without touching AWS.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/spire-install-parallel.XXXXXX")"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

bin_dir="$work_dir/bin"
artifact_dir="$work_dir/artifacts"
topology="$work_dir/topology.json"
events="$work_dir/events.log"
mkdir -p "$bin_dir" "$artifact_dir"
: > "$events"
printf 'source fixture\n' > "$artifact_dir/ecaz-source.tar.gz"

cat > "$topology" <<'JSON'
{
  "region": "us-west-2",
  "artifact_bucket": "ecaz-spire-aws-test",
  "coordinator": {
    "instance_id": "i-coord",
    "private_ip": "10.42.1.10"
  },
  "remotes": [
    {
      "node_id": 2,
      "instance_id": "i-remote-1",
      "private_ip": "10.42.1.11",
      "secret_name": "remote-1"
    },
    {
      "node_id": 3,
      "instance_id": "i-remote-2",
      "private_ip": "10.42.1.12",
      "secret_name": "remote-2"
    },
    {
      "node_id": 4,
      "instance_id": "i-remote-3",
      "private_ip": "10.42.1.13",
      "secret_name": "remote-3"
    }
  ]
}
JSON

cat > "$bin_dir/aws" <<'BASH'
#!/usr/bin/env bash
set -euo pipefail

events="${SPIRE_AWS_INSTALL_PARALLEL_EVENTS:?events path required}"
service="${1:?service required}"
operation="${2:?operation required}"
shift 2

arg_value() {
  local key="$1"
  shift
  while (($#)); do
    if [[ "$1" == "$key" ]]; then
      shift
      printf '%s\n' "${1:-}"
      return 0
    fi
    shift
  done
  return 1
}

has_arg() {
  local key="$1"
  shift
  while (($#)); do
    if [[ "$1" == "$key" ]]; then
      return 0
    fi
    shift
  done
  return 1
}

case "${service}:${operation}" in
  ssm:describe-instance-information)
    printf 'Online\n'
    ;;
  s3:cp)
    printf 'copied\n'
    ;;
  secretsmanager:get-secret-value)
    printf '{"host":"127.0.0.1","port":5432,"dbname":"postgres","user":"postgres","password":"pw","sslmode":"verify-full","sslrootcert":"/var/lib/pgsql/ca.crt"}\n'
    ;;
  ssm:send-command)
    instance_id="$(arg_value --instance-ids "$@")"
    comment="$(arg_value --comment "$@" || true)"
    parameters="$(arg_value --parameters "$@")"
    if [[ "$comment" == *"coordinator remote conninfo"* ]]; then
      printf 'conninfo-send %s\n' "$instance_id" >> "$events"
      printf 'cmd-conninfo-%s\n' "$instance_id"
    else
      command_line="$(jq -r '.commands[1]' <<< "$parameters")"
      source_key="$(grep -o 'ECAZ_SPIRE_AWS_SOURCE_KEY=[^ ]*' <<< "$command_line" | cut -d= -f2-)"
      build_runtime="$(grep -o 'ECAZ_SPIRE_AWS_BUILD_RUNTIME=[^ ]*' <<< "$command_line" | cut -d= -f2-)"
      wait_runtime="$(grep -o 'ECAZ_SPIRE_AWS_WAIT_RUNTIME=[^ ]*' <<< "$command_line" | cut -d= -f2-)"
      runtime_key="$(grep -o 'ECAZ_SPIRE_AWS_RUNTIME_KEY=[^ ]*' <<< "$command_line" | cut -d= -f2-)"
      printf 'install-send %s\n' "$instance_id" >> "$events"
      printf 'install-mode %s source=%s runtime=%s build=%s wait=%s\n' \
        "$instance_id" "$source_key" "$runtime_key" "$build_runtime" "$wait_runtime" >> "$events"
      printf 'cmd-install-%s\n' "$instance_id"
    fi
    ;;
  ssm:get-command-invocation)
    instance_id="$(arg_value --instance-id "$@")"
    cmd_id="$(arg_value --command-id "$@")"
    if [[ "$cmd_id" == cmd-install-* ]]; then
      if has_arg --query "$@"; then
        printf 'install-wait %s\n' "$instance_id" >> "$events"
        printf 'Success\n'
      else
        printf 'install-fetch %s\n' "$instance_id" >> "$events"
        printf '{"Status":"Success","CommandId":"%s","InstanceId":"%s"}\n' "$cmd_id" "$instance_id"
      fi
    else
      if has_arg --query "$@"; then
        printf 'conninfo-wait %s\n' "$instance_id" >> "$events"
        printf 'Success\n'
      else
        printf 'conninfo-fetch %s\n' "$instance_id" >> "$events"
        printf '{"Status":"Success","CommandId":"%s","InstanceId":"%s"}\n' "$cmd_id" "$instance_id"
      fi
    fi
    ;;
  *)
    printf 'unexpected aws call: %s %s %s\n' "$service" "$operation" "$*" >&2
    exit 64
    ;;
esac
BASH

cat > "$bin_dir/openssl" <<'BASH'
#!/usr/bin/env bash
set -euo pipefail

out=""
while (($#)); do
  if [[ "$1" == "-out" ]]; then
    shift
    out="${1:-}"
  fi
  shift || true
done

if [[ -n "$out" ]]; then
  mkdir -p "$(dirname "$out")"
  printf 'stub\n' > "$out"
fi
BASH

chmod +x "$bin_dir/aws" "$bin_dir/openssl"

PATH="$bin_dir:$PATH" \
SPIRE_AWS_INSTALL_PARALLEL_EVENTS="$events" \
scripts/spire-aws/install.sh "$topology" "$artifact_dir" > "$artifact_dir/install-selfcheck.stdout" 2> "$artifact_dir/install-selfcheck.stderr"

install_sends="$(grep -c '^install-send ' "$events")"
install_waits="$(grep -c '^install-wait ' "$events")"
first_wait_line="$(grep -n '^install-wait ' "$events" | head -1 | cut -d: -f1)"
last_send_line="$(grep -n '^install-send ' "$events" | tail -1 | cut -d: -f1)"

if [[ "$install_sends" != "4" ]]; then
  printf 'ERROR: expected 4 install-send events, got %s\n' "$install_sends" >&2
  cat "$events" >&2
  exit 1
fi

if [[ "$install_waits" != "4" ]]; then
  printf 'ERROR: expected 4 install-wait events, got %s\n' "$install_waits" >&2
  cat "$events" >&2
  exit 1
fi

if (( last_send_line >= first_wait_line )); then
  printf 'ERROR: install wait started before all install sends completed\n' >&2
  cat "$events" >&2
  exit 1
fi

if ! grep -q '^conninfo-send i-coord$' "$events"; then
  printf 'ERROR: coordinator conninfo command was not reached\n' >&2
  cat "$events" >&2
  exit 1
fi

if ! grep -q '^install-mode i-coord source=ecaz-source.tar.gz runtime=ecaz-runtime-linux-aarch64.tar.gz build=1 wait=0$' "$events"; then
  printf 'ERROR: coordinator install mode did not build and publish runtime\n' >&2
  cat "$events" >&2
  exit 1
fi

for remote in i-remote-1 i-remote-2 i-remote-3; do
  if ! grep -q "^install-mode ${remote} source= runtime=ecaz-runtime-linux-aarch64.tar.gz build=0 wait=1$" "$events"; then
    printf 'ERROR: remote install mode did not wait for coordinator runtime: %s\n' "$remote" >&2
    cat "$events" >&2
    exit 1
  fi
done

cat "$events"
printf 'SPIRE AWS install parallel self-check passed: install_sends=%s install_waits=%s\n' \
  "$install_sends" "$install_waits"
