#!/usr/bin/env bash
# Phase 13b.5 — runs once on every coordinator/remote node via SSM.
# Installs PostgreSQL 18, the ecaz extension tarball from S3, and sets
# the load-bearing Phase 13a.1.b GUCs. Idempotent.

set -euo pipefail

PG_VERSION=18
BUCKET="${ECAZ_SPIRE_AWS_BUCKET:?bucket must be set by SSM document}"
ECAZ_KEY="${ECAZ_SPIRE_AWS_TARBALL_KEY:-ecaz-latest.tar.gz}"
SOURCE_KEY="${ECAZ_SPIRE_AWS_SOURCE_KEY:-}"
TLS_KEY="${ECAZ_SPIRE_AWS_TLS_KEY:?TLS material key must be set by SSM document}"
REMOTE_SECRET_NAME="${ECAZ_SPIRE_AWS_REMOTE_SECRET_NAME:-}"
REGION="${ECAZ_SPIRE_AWS_REGION:?region must be set by SSM document}"

dnf -y install postgresql${PG_VERSION}-server postgresql${PG_VERSION}-contrib postgresql${PG_VERSION}-server-devel jq awscli openssl

if [[ -x "/usr/pgsql-${PG_VERSION}/bin/postgres" ]]; then
  PG_BIN="/usr/pgsql-${PG_VERSION}/bin"
  PGDATA=/var/lib/pgsql/${PG_VERSION}/data
  PG_SERVICE="postgresql-${PG_VERSION}"
else
  PG_BIN="/usr/bin"
  PGDATA=/var/lib/pgsql/data
  PG_SERVICE="postgresql"
fi

if [[ ! -s "${PGDATA}/PG_VERSION" ]]; then
  if [[ -x "/usr/pgsql-${PG_VERSION}/bin/postgresql-${PG_VERSION}-setup" ]]; then
    "/usr/pgsql-${PG_VERSION}/bin/postgresql-${PG_VERSION}-setup" initdb || true
  elif command -v "postgresql-${PG_VERSION}-setup" >/dev/null 2>&1; then
    "postgresql-${PG_VERSION}-setup" initdb || true
  elif command -v postgresql-setup >/dev/null 2>&1; then
    postgresql-setup --initdb || postgresql-setup --initdb --unit "$PG_SERVICE" || true
  fi
fi

if [[ ! -s "${PGDATA}/PG_VERSION" ]]; then
  install -o postgres -g postgres -m 0700 -d "$PGDATA"
  sudo -u postgres "${PG_BIN}/initdb" -D "$PGDATA"
fi

aws s3 cp "s3://${BUCKET}/${TLS_KEY}" /tmp/ecaz-node-tls.tar.gz
mkdir -p /tmp/ecaz-node-tls
tar -xzf /tmp/ecaz-node-tls.tar.gz -C /tmp/ecaz-node-tls
install -o postgres -g postgres -m 0600 /tmp/ecaz-node-tls/server.key "${PGDATA}/server.key"
install -o postgres -g postgres -m 0644 /tmp/ecaz-node-tls/server.crt "${PGDATA}/server.crt"
install -o root -g root -m 0644 /tmp/ecaz-node-tls/ca.crt /etc/ssl/certs/ecaz-spire-aws-ca.pem

mem_total_kb=$(awk '/MemTotal:/ { print $2 }' /proc/meminfo)
mem_total_gb=$((mem_total_kb / 1024 / 1024))
if ((mem_total_gb >= 96)); then
  shared_buffers="${ECAZ_SPIRE_AWS_SHARED_BUFFERS:-32GB}"
  maintenance_work_mem="${ECAZ_SPIRE_AWS_MAINTENANCE_WORK_MEM:-2GB}"
elif ((mem_total_gb >= 32)); then
  shared_buffers="${ECAZ_SPIRE_AWS_SHARED_BUFFERS:-8GB}"
  maintenance_work_mem="${ECAZ_SPIRE_AWS_MAINTENANCE_WORK_MEM:-1GB}"
else
  shared_buffers="${ECAZ_SPIRE_AWS_SHARED_BUFFERS:-2GB}"
  maintenance_work_mem="${ECAZ_SPIRE_AWS_MAINTENANCE_WORK_MEM:-512MB}"
fi
work_mem="${ECAZ_SPIRE_AWS_WORK_MEM:-64MB}"

cat >> "${PGDATA}/postgresql.conf" <<EOF
listen_addresses = '*'
shared_buffers = ${shared_buffers}
work_mem = ${work_mem}
maintenance_work_mem = ${maintenance_work_mem}
max_prepared_transactions = 64
shared_preload_libraries = 'ecaz'
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
EOF

tmp_hba=$(mktemp)
cat > "$tmp_hba" <<EOF
host all ecaz_coord 127.0.0.1/32 trust
host all ecaz_coord ::1/128 trust
hostssl all ecaz_coord 10.42.0.0/16 scram-sha-256
EOF
grep -vE 'ecaz_coord (127\.0\.0\.1/32|::1/128|10\.42\.0\.0/16)' "${PGDATA}/pg_hba.conf" >> "$tmp_hba"
install -o postgres -g postgres -m 0600 "$tmp_hba" "${PGDATA}/pg_hba.conf"
rm -f "$tmp_hba"

aws s3 cp "s3://${BUCKET}/${ECAZ_KEY}" /tmp/ecaz.tar.gz
mkdir -p /tmp/ecaz && tar -xzf /tmp/ecaz.tar.gz -C /tmp/ecaz
if [[ -x "${PG_BIN}/pg_config" ]]; then
  PKGLIBDIR=$("${PG_BIN}/pg_config" --pkglibdir)
  SHAREDIR=$("${PG_BIN}/pg_config" --sharedir)
elif command -v pg_config >/dev/null 2>&1; then
  PKGLIBDIR=$(pg_config --pkglibdir)
  SHAREDIR=$(pg_config --sharedir)
else
  PKGLIBDIR="/usr/lib64/pgsql"
  SHAREDIR="/usr/share/pgsql"
fi
EXTENSION_DIR="${SHAREDIR}/extension"
install -d "$PKGLIBDIR" "$EXTENSION_DIR"
cp /tmp/ecaz/extension/* "$EXTENSION_DIR/"

if [[ -n "$SOURCE_KEY" ]]; then
  dnf -y install rust cargo rustfmt gcc gcc-c++ make clang
  aws s3 cp "s3://${BUCKET}/${SOURCE_KEY}" /tmp/ecaz-source.tar.gz
  rm -rf /tmp/ecaz-source && mkdir -p /tmp/ecaz-source
  tar -xzf /tmp/ecaz-source.tar.gz -C /tmp/ecaz-source
  (
    cd /tmp/ecaz-source
    PG_CONFIG="${PG_BIN}/pg_config" \
      PGRX_PG_CONFIG_PATH="${PG_BIN}/pg_config" \
      CARGO_PROFILE_RELEASE_LTO=thin \
      CARGO_PROFILE_RELEASE_CODEGEN_UNITS=4 \
      cargo build --release --lib --package ecaz --no-default-features --features pg18 --offline
    PG_CONFIG="${PG_BIN}/pg_config" \
      PGRX_PG_CONFIG_PATH="${PG_BIN}/pg_config" \
      CARGO_PROFILE_RELEASE_LTO=thin \
      CARGO_PROFILE_RELEASE_CODEGEN_UNITS=4 \
      cargo build --release --bin ecaz --package ecaz-cli --offline
  )
  install -m 0755 /tmp/ecaz-source/target/release/libecaz.so "${PKGLIBDIR}/ecaz.so"
  install -m 0755 /tmp/ecaz-source/target/release/ecaz /usr/local/bin/ecaz
else
  cp /tmp/ecaz/lib/*.so "${PKGLIBDIR}/"
fi

systemctl enable --now "$PG_SERVICE"
systemctl restart "$PG_SERVICE"

ROLE_PASSWORD=""
if [[ -n "$REMOTE_SECRET_NAME" ]]; then
  ROLE_PASSWORD=$(aws secretsmanager get-secret-value \
    --region "$REGION" \
    --secret-id "$REMOTE_SECRET_NAME" \
    --query SecretString \
    --output text | jq -r '.password')
fi

if [[ -n "$ROLE_PASSWORD" ]]; then
  sudo -u postgres "${PG_BIN}/psql" -v ON_ERROR_STOP=1 -v role_password="$ROLE_PASSWORD" <<'SQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ecaz_coord') THEN
    CREATE ROLE ecaz_coord LOGIN SUPERUSER;
  END IF;
END $$;
ALTER ROLE ecaz_coord WITH LOGIN SUPERUSER PASSWORD :'role_password';
SQL
else
  sudo -u postgres "${PG_BIN}/psql" -v ON_ERROR_STOP=1 <<'SQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ecaz_coord') THEN
    CREATE ROLE ecaz_coord LOGIN SUPERUSER;
  END IF;
END $$;
SQL
fi

sudo -u postgres "${PG_BIN}/psql" -c "CREATE EXTENSION IF NOT EXISTS ecaz" || true
sudo -u postgres "${PG_BIN}/psql" -c "SELECT extversion FROM pg_extension WHERE extname='ecaz'"
