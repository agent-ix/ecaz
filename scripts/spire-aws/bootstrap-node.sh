#!/usr/bin/env bash
# Phase 13b.5 — runs once on every coordinator/remote node via SSM.
# Installs PostgreSQL 18, the ecaz extension tarball from S3, and sets
# the load-bearing Phase 13a.1.b GUCs. Idempotent.

set -euxo pipefail

PG_VERSION=18
BUCKET="${ECAZ_SPIRE_AWS_BUCKET:?bucket must be set by SSM document}"
ECAZ_KEY="${ECAZ_SPIRE_AWS_TARBALL_KEY:-ecaz-latest.tar.gz}"

dnf -y install postgresql${PG_VERSION}-server postgresql${PG_VERSION}-contrib jq awscli

/usr/pgsql-${PG_VERSION}/bin/postgresql-${PG_VERSION}-setup initdb || true

PGDATA=/var/lib/pgsql/${PG_VERSION}/data
cat >> "${PGDATA}/postgresql.conf" <<EOF
shared_buffers = 32GB
work_mem = 64MB
maintenance_work_mem = 2GB
max_prepared_transactions = 64
shared_preload_libraries = 'ecaz'
ssl = on
EOF

aws s3 cp "s3://${BUCKET}/${ECAZ_KEY}" /tmp/ecaz.tar.gz
mkdir -p /tmp/ecaz && tar -xzf /tmp/ecaz.tar.gz -C /tmp/ecaz
cp /tmp/ecaz/lib/*.so "/usr/pgsql-${PG_VERSION}/lib/"
cp /tmp/ecaz/extension/* "/usr/pgsql-${PG_VERSION}/share/extension/"

systemctl enable --now "postgresql-${PG_VERSION}"
systemctl restart "postgresql-${PG_VERSION}"

sudo -u postgres /usr/pgsql-${PG_VERSION}/bin/psql -c "CREATE EXTENSION IF NOT EXISTS ecaz" || true
sudo -u postgres /usr/pgsql-${PG_VERSION}/bin/psql -c "SELECT extversion FROM pg_extension WHERE extname='ecaz'"
