#!/usr/bin/env bash
# Phase 13b.5 — runs once on every coordinator/remote node via SSM.
# Installs PostgreSQL 18, the ecaz extension tarball from S3, and sets
# the load-bearing Phase 13a.1.b GUCs. Idempotent.

set -euxo pipefail

PG_VERSION=18
BUCKET="${ECAZ_SPIRE_AWS_BUCKET:?bucket must be set by SSM document}"
ECAZ_KEY="${ECAZ_SPIRE_AWS_TARBALL_KEY:-ecaz-latest.tar.gz}"
TLS_KEY="${ECAZ_SPIRE_AWS_TLS_KEY:?TLS material key must be set by SSM document}"
REMOTE_SECRET_NAME="${ECAZ_SPIRE_AWS_REMOTE_SECRET_NAME:-}"
REGION="${ECAZ_SPIRE_AWS_REGION:?region must be set by SSM document}"

dnf -y install postgresql${PG_VERSION}-server postgresql${PG_VERSION}-contrib jq awscli openssl

/usr/pgsql-${PG_VERSION}/bin/postgresql-${PG_VERSION}-setup initdb || true

PGDATA=/var/lib/pgsql/${PG_VERSION}/data
aws s3 cp "s3://${BUCKET}/${TLS_KEY}" /tmp/ecaz-node-tls.tar.gz
mkdir -p /tmp/ecaz-node-tls
tar -xzf /tmp/ecaz-node-tls.tar.gz -C /tmp/ecaz-node-tls
install -o postgres -g postgres -m 0600 /tmp/ecaz-node-tls/server.key "${PGDATA}/server.key"
install -o postgres -g postgres -m 0644 /tmp/ecaz-node-tls/server.crt "${PGDATA}/server.crt"
install -o root -g root -m 0644 /tmp/ecaz-node-tls/ca.crt /etc/ssl/certs/ecaz-spire-aws-ca.pem

cat >> "${PGDATA}/postgresql.conf" <<EOF
listen_addresses = '*'
shared_buffers = 32GB
work_mem = 64MB
maintenance_work_mem = 2GB
max_prepared_transactions = 64
shared_preload_libraries = 'ecaz'
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
EOF

cat >> "${PGDATA}/pg_hba.conf" <<EOF
host all ecaz_coord 127.0.0.1/32 trust
host all ecaz_coord ::1/128 trust
hostssl all ecaz_coord 10.42.0.0/16 scram-sha-256
EOF

aws s3 cp "s3://${BUCKET}/${ECAZ_KEY}" /tmp/ecaz.tar.gz
mkdir -p /tmp/ecaz && tar -xzf /tmp/ecaz.tar.gz -C /tmp/ecaz
cp /tmp/ecaz/lib/*.so "/usr/pgsql-${PG_VERSION}/lib/"
cp /tmp/ecaz/extension/* "/usr/pgsql-${PG_VERSION}/share/extension/"

systemctl enable --now "postgresql-${PG_VERSION}"
systemctl restart "postgresql-${PG_VERSION}"

ROLE_PASSWORD=""
if [[ -n "$REMOTE_SECRET_NAME" ]]; then
  ROLE_PASSWORD=$(aws secretsmanager get-secret-value \
    --region "$REGION" \
    --secret-id "$REMOTE_SECRET_NAME" \
    --query SecretString \
    --output text | jq -r '.password')
fi

if [[ -n "$ROLE_PASSWORD" ]]; then
  sudo -u postgres /usr/pgsql-${PG_VERSION}/bin/psql -v ON_ERROR_STOP=1 -v role_password="$ROLE_PASSWORD" <<'SQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ecaz_coord') THEN
    CREATE ROLE ecaz_coord LOGIN SUPERUSER;
  END IF;
END $$;
ALTER ROLE ecaz_coord WITH LOGIN SUPERUSER PASSWORD :'role_password';
SQL
else
  sudo -u postgres /usr/pgsql-${PG_VERSION}/bin/psql -v ON_ERROR_STOP=1 <<'SQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ecaz_coord') THEN
    CREATE ROLE ecaz_coord LOGIN SUPERUSER;
  END IF;
END $$;
SQL
fi

sudo -u postgres /usr/pgsql-${PG_VERSION}/bin/psql -c "CREATE EXTENSION IF NOT EXISTS ecaz" || true
sudo -u postgres /usr/pgsql-${PG_VERSION}/bin/psql -c "SELECT extversion FROM pg_extension WHERE extname='ecaz'"
