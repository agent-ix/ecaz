#!/usr/bin/env bash
# Phase 13b.5 — install PostgreSQL 18 and the ecaz extension on every node.
# Args:
#   $1  Path to topology JSON (from `terraform output -json topology`)
#   $2  Artifact directory for logs
#
# Uses AWS Session Manager (`aws ssm send-command`) to run the bootstrap
# script on every instance in parallel. Each node receives the ecaz tarball
# from S3 and writes its install transcript back to the artifact bucket.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
BUCKET=$(jq -r '.artifact_bucket' "$TOPOLOGY")
COORD_ID=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")
REMOTE_IDS=$(jq -r '.remotes[].instance_id' "$TOPOLOGY")
TARBALL_KEY="${ECAZ_SPIRE_AWS_TARBALL_KEY:-ecaz-latest.tar.gz}"
TARBALL_PATH="${ECAZ_SPIRE_AWS_TARBALL_PATH:-}"
TLS_DIR="$ARTIFACT_DIR/tls"

ALL_IDS=("$COORD_ID")
while IFS= read -r id; do ALL_IDS+=("$id"); done <<< "$REMOTE_IDS"

aws s3 cp \
  "$REPO_ROOT/scripts/spire-aws/bootstrap-node.sh" \
  "s3://${BUCKET}/bootstrap-node.sh" \
  --region "$REGION" \
  > "$ARTIFACT_DIR/bootstrap-upload.log"

if [[ -n "$TARBALL_PATH" ]]; then
  aws s3 cp \
    "$TARBALL_PATH" \
    "s3://${BUCKET}/${TARBALL_KEY}" \
    --region "$REGION" \
    > "$ARTIFACT_DIR/tarball-upload.log"
fi

mkdir -p "$TLS_DIR"
if [[ ! -s "$TLS_DIR/ca.key" || ! -s "$TLS_DIR/ca.crt" ]]; then
  openssl genrsa -out "$TLS_DIR/ca.key" 2048
  openssl req -x509 -new -nodes -key "$TLS_DIR/ca.key" -sha256 -days 7 \
    -subj "/CN=ecaz-spire-aws-ca" \
    -out "$TLS_DIR/ca.crt"
fi

write_tls_bundle() {
  local instance_id="$1"
  local private_ip="$2"
  local node_dir="$TLS_DIR/${instance_id}"
  local ext_file="$node_dir/server.ext"
  local bundle="$TLS_DIR/${instance_id}.tar.gz"

  mkdir -p "$node_dir"
  openssl genrsa -out "$node_dir/server.key" 2048
  openssl req -new -key "$node_dir/server.key" \
    -subj "/CN=${private_ip}" \
    -out "$node_dir/server.csr"
  cat > "$ext_file" <<EOF
subjectAltName = IP:${private_ip}
extendedKeyUsage = serverAuth
EOF
  openssl x509 -req \
    -in "$node_dir/server.csr" \
    -CA "$TLS_DIR/ca.crt" \
    -CAkey "$TLS_DIR/ca.key" \
    -CAcreateserial \
    -out "$node_dir/server.crt" \
    -days 7 \
    -sha256 \
    -extfile "$ext_file"
  cp "$TLS_DIR/ca.crt" "$node_dir/ca.crt"
  tar -C "$node_dir" -czf "$bundle" server.key server.crt ca.crt
  aws s3 cp "$bundle" "s3://${BUCKET}/tls/${instance_id}.tar.gz" --region "$REGION" \
    > "$ARTIFACT_DIR/tls-upload-${instance_id}.log"
}

send_install_command() {
  local instance_id="$1"
  local secret_name="$2"
  local commands_json
  local parameters_json
  local cmd_id

  commands_json=$(jq -cn \
    --arg bucket "$BUCKET" \
    --arg tarball_key "$TARBALL_KEY" \
    --arg tls_key "tls/${instance_id}.tar.gz" \
    --arg secret_name "$secret_name" \
    --arg region "$REGION" \
    '[
      "sudo aws s3 cp s3://\($bucket)/bootstrap-node.sh /tmp/bootstrap-node.sh",
      "sudo ECAZ_SPIRE_AWS_BUCKET=\($bucket) ECAZ_SPIRE_AWS_TARBALL_KEY=\($tarball_key) ECAZ_SPIRE_AWS_TLS_KEY=\($tls_key) ECAZ_SPIRE_AWS_REMOTE_SECRET_NAME=\($secret_name) ECAZ_SPIRE_AWS_REGION=\($region) bash /tmp/bootstrap-node.sh"
    ]')
  parameters_json=$(jq -cn --argjson commands "$commands_json" '{commands: $commands}')

  cmd_id=$(aws ssm send-command \
    --region "$REGION" \
    --document-name "AWS-RunShellScript" \
    --instance-ids "$instance_id" \
    --parameters "$parameters_json" \
    --output-s3-bucket-name "$BUCKET" \
    --output-s3-key-prefix "spire-aws/install/${instance_id}" \
    --comment "ecaz Phase 13b.5 install ${instance_id}" \
    --query "Command.CommandId" --output text)

  echo "${instance_id} ssm command id: ${cmd_id}" | tee -a "$ARTIFACT_DIR/install.log"
  aws ssm wait command-executed --region "$REGION" --command-id "$cmd_id" --instance-id "$instance_id"
  aws ssm get-command-invocation \
    --region "$REGION" --command-id "$cmd_id" --instance-id "$instance_id" \
    > "$ARTIFACT_DIR/install-${instance_id}.log"
}

coord_ip=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")
write_tls_bundle "$COORD_ID" "$coord_ip"
send_install_command "$COORD_ID" ""

jq -c '.remotes[]' "$TOPOLOGY" | while read -r remote; do
  instance_id=$(jq -r '.instance_id' <<< "$remote")
  private_ip=$(jq -r '.private_ip' <<< "$remote")
  secret_name=$(jq -r '.secret_name' <<< "$remote")
  write_tls_bundle "$instance_id" "$private_ip"
  send_install_command "$instance_id" "$secret_name"
done
