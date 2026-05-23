#!/usr/bin/env bash
# Phase 13b.5 — runs once on every coordinator/remote node via SSM.
# Installs PostgreSQL 18 from Amazon Linux 2023's native repos, builds the
# ecaz extension from a git ref via cargo pgrx, sets the load-bearing
# Phase 13a.1.b GUCs, and CREATEs the extension. Idempotent.
#
# Pattern mirrors infra/cloud/terraform/cloud-init/db.sh.tftpl which has
# been the proven Graviton 4 install path since the Graviton baseline
# cycle (2026-05-16). No PGDG repo, no S3 tarball staging: each node
# clones the ecaz repo and runs `cargo pgrx install --release` natively.
#
# Required env (set by SSM document parameters from install.sh):
#   ECAZ_SPIRE_AWS_BUCKET   S3 bucket for log/artifact upload (unused for
#                           binary distribution; binaries are built locally).
#   ECAZ_GIT_URL            git URL to clone (default: GitHub canonical).
#   ECAZ_GIT_REF            git ref / SHA to check out (REQUIRED).

set -euxo pipefail

PG_VERSION=18
BUCKET="${ECAZ_SPIRE_AWS_BUCKET:?bucket must be set by SSM document}"
ECAZ_URL="${ECAZ_GIT_URL:-https://github.com/agent-ix/ecaz.git}"
ECAZ_REF="${ECAZ_GIT_REF:?git ref must be set by SSM document}"

# --- System packages (PG18 ships in AL2023 native repos) -------------------
dnf -y update
dnf -y install --allowerasing \
  git gcc gcc-c++ make clang clang-devel llvm llvm-devel \
  openssl-devel readline-devel zlib-devel bzip2-devel \
  jq cmake pkgconfig perl
dnf -y install postgresql${PG_VERSION}-server postgresql${PG_VERSION}-contrib postgresql${PG_VERSION}-server-devel

# --- Postgres initdb + GUC tuning ------------------------------------------
# AL2023 PG18 layout: binaries in /usr/bin/, default PGDATA at
# /var/lib/pgsql/data. Match cloud-init's convention by keeping PGDATA at
# /var/lib/pgsql/18/data for parity with the existing Graviton lane.
PGDATA=/var/lib/pgsql/${PG_VERSION}/data
PG_BIN_DIR=/usr/bin
PG_CONFIG=/usr/bin/pg_config

mkdir -p /var/lib/pgsql/${PG_VERSION}
chown -R postgres:postgres /var/lib/pgsql

if [ ! -s "${PGDATA}/PG_VERSION" ]; then
  sudo -u postgres "${PG_BIN_DIR}/initdb" -D "${PGDATA}" --locale=C --encoding=UTF8
fi

# Phase 13a.1.b GUCs. shared_preload_libraries='ecaz' is appended AFTER
# the cargo pgrx install step below; PG must not start with it set until
# ecaz.so exists.
# F20: scale shared_buffers to instance RAM. 32 GB hardcoded value
# was sized for the spec r8g.4xlarge (128 GB); fails on smaller nodes
# (r8g.xlarge=32 GB, r8g.large=16 GB) which can't map that much
# anonymous shared memory.
#
# Rule of thumb: shared_buffers = 25% of total RAM, capped at 8 GB
# (returns diminishing perf above that for our workload).
TOTAL_MEM_KB=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
SHARED_MB=$(( TOTAL_MEM_KB / 1024 / 4 ))
if [ "$SHARED_MB" -gt 8192 ]; then SHARED_MB=8192; fi
if [ "$SHARED_MB" -lt 256 ]; then SHARED_MB=256; fi

cat > "${PGDATA}/postgresql.auto.conf" <<EOF
listen_addresses = '*'
port = 5432
shared_buffers = ${SHARED_MB}MB
work_mem = 64MB
maintenance_work_mem = 512MB
max_prepared_transactions = 64
ssl = on
EOF

# ssl=on requires server.crt + server.key in $PGDATA. Generate a
# self-signed cert valid for 1 day (enough for a packet run) so PG can
# start. Production deployments should replace with a real cert; this is
# a smoke-test setup.
sudo -u postgres bash -c "
  cd \"${PGDATA}\"
  if [ ! -s server.crt ]; then
    openssl req -new -x509 -days 1 -nodes -text \
      -out server.crt -keyout server.key \
      -subj '/CN=ecaz-spire-aws-smoke'
    chmod 600 server.key server.crt
  fi
"

cat > "${PGDATA}/pg_hba.conf" <<EOF
local all all                     trust
host  all all 127.0.0.1/32        trust
host  all all 10.42.0.0/16        trust
EOF

mkdir -p /etc/systemd/system/postgresql.service.d
cat > /etc/systemd/system/postgresql.service.d/pgdata.conf <<EOF
[Service]
Environment=PGDATA=${PGDATA}
EOF
systemctl daemon-reload
systemctl enable postgresql
systemctl start postgresql

# --- Rust + cargo-pgrx as the postgres user --------------------------------
# pgrx install --sudo writes ecaz.control + ecaz.so into /usr/share and
# /usr/lib64 paths owned by root; passwordless sudo for postgres lets the
# single command do both build and install steps.
echo 'postgres ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/postgres
chmod 440 /etc/sudoers.d/postgres

sudo -u postgres bash -c '
  set -eux
  if ! [ -x "$HOME/.cargo/bin/cargo" ]; then
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain stable --profile minimal
  fi
  # shellcheck disable=SC1091
  . $HOME/.cargo/env
  cargo install --locked cargo-pgrx@^0.17
  cargo pgrx init --pg18 /usr/bin/pg_config
'

# --- Clone + build + install ecaz at the requested git ref -----------------
sudo -u postgres bash -lc "
  set -eux
  export PATH=\$HOME/.cargo/bin:\$PATH
  # F18: cap release-profile LTO to 'thin' so the build fits in 16 GB
  # (release default is lto='fat'+codegen-units=1 which peaks at 24 GB,
  # OOMs the r8g.large remote). Trades a small runtime perf hit for
  # being able to build under the current vCPU/RAM quota allowance.
  # Applied to every node for binary determinism.
  export CARGO_PROFILE_RELEASE_LTO=thin
  export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=4
  if [ ! -d /var/lib/pgsql/build/ecaz/.git ]; then
    rm -rf /var/lib/pgsql/build
    mkdir -p /var/lib/pgsql/build
    # F23: shallow-clone the default branch then fetch the specific
    # commit. \`git fetch origin <sha>\` works against GitHub since
    # uploadpack.allowReachableSHA1InWant is enabled.
    git clone --filter=blob:none ${ECAZ_URL} /var/lib/pgsql/build/ecaz
  fi
  cd /var/lib/pgsql/build/ecaz
  git fetch origin ${ECAZ_REF}
  git checkout FETCH_HEAD
  cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config
  # ecaz CLI on the node for in-VPC corpus/bench operations.
  cargo build --release -p ecaz-cli
"
install -Dm755 /var/lib/pgsql/build/ecaz/target/release/ecaz /usr/local/bin/ecaz

# Now that ecaz.so exists, enable shared_preload_libraries and restart PG.
cat >> "${PGDATA}/postgresql.auto.conf" <<EOF
shared_preload_libraries = 'ecaz'
EOF
systemctl restart postgresql
sleep 3

sudo -u postgres psql -c 'CREATE EXTENSION IF NOT EXISTS ecaz;'
sudo -u postgres psql -c "SELECT extname, extversion FROM pg_extension WHERE extname = 'ecaz';"

# Create the ecaz_coord role expected by scripts/spire-aws/*.sh. Superuser
# because the SPIRE coordinator routines require server-level privileges
# (register remotes, manage secrets bindings, etc.). Idempotent.
sudo -u postgres psql <<'PSQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ecaz_coord') THEN
    CREATE ROLE ecaz_coord WITH LOGIN SUPERUSER;
  END IF;
END
$$;
PSQL
