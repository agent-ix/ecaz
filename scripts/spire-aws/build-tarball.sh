#!/usr/bin/env bash
# Build ecaz extension tarball for Amazon Linux 2023 / aarch64 (Graviton 4).
# Intended to run INSIDE an `amazonlinux:2023` aarch64 container; expects
# /work to be the repo root mounted from the host.
#
# Target hardware: AWS Graviton 4 (Neoverse V2 cores, armv9-a + SVE2).
# Forward-compatible with Graviton 5 (V2-or-newer expected). NOT compatible
# with Graviton 2/3 — those would need a separate `target-cpu` build.
#
# Output: target/ecaz-spire-aws-<short-sha>.tar.gz
# Layout inside tarball matches what bootstrap-node.sh consumes:
#   lib/ecaz.so
#   extension/ecaz.control + ecaz--<ver>.sql

set -euxo pipefail

# Sanity: refuse to run on the wrong arch — produces silently bad binaries.
if [ "$(uname -m)" != "aarch64" ]; then
  echo "ERROR: build-tarball.sh must run on aarch64 (uname -m = $(uname -m))" >&2
  echo "       Run via 'make package' which sets --platform linux/arm64." >&2
  exit 2
fi

PG_VERSION=18
WORK_DIR="${WORK_DIR:-/work}"
cd "$WORK_DIR"

# PGDG repo + PG18 server/dev (AL2023 = EL9-compatible, aarch64).
# AL2023 lacks /etc/redhat-release; PGDG's RPM requires it. Fake just
# enough for the rpm preinstall scripts to accept us as RHEL9.
if [ ! -f /etc/redhat-release ]; then
  echo "Red Hat Enterprise Linux release 9.4 (Plow)" > /etc/redhat-release
fi
dnf -y install https://download.postgresql.org/pub/repos/yum/reporpms/EL-9-aarch64/pgdg-redhat-repo-latest.noarch.rpm
dnf -qy module disable postgresql || true
dnf -y install \
  postgresql${PG_VERSION}-server postgresql${PG_VERSION}-contrib postgresql${PG_VERSION}-devel \
  gcc gcc-c++ make clang clang-devel llvm-devel openssl-devel pkgconfig \
  git curl tar gzip ca-certificates which findutils

export PATH=/usr/pgsql-${PG_VERSION}/bin:/root/.cargo/bin:$PATH

# Rust toolchain (cached via volume mount on /root/.cargo + /root/.rustup).
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable --profile minimal
fi
# shellcheck disable=SC1091
. /root/.cargo/env

# cargo-pgrx must match what the workspace expects.
PGRX_VERSION="$(awk '/^name = "pgrx"/{p=1;next} p && /^version =/{gsub(/[",]/,"",$3); print $3; exit}' Cargo.lock)"
PGRX_VERSION="${PGRX_VERSION:-0.17.0}"
if ! cargo pgrx --version 2>/dev/null | grep -q "$PGRX_VERSION"; then
  cargo install --locked --force "cargo-pgrx@${PGRX_VERSION}"
fi

# Initialize pgrx against the system PG18 install (not pgrx-managed).
cargo pgrx init --pg${PG_VERSION} "/usr/pgsql-${PG_VERSION}/bin/pg_config"

# Tune codegen for Graviton 4 (Neoverse V2 / armv9-a / SVE2). G5 should be
# forward-compatible. NOT compatible with G2/G3 — change target-cpu if
# retargeting.
export RUSTFLAGS="${RUSTFLAGS:-} -C target-cpu=neoverse-v2"
export CFLAGS="${CFLAGS:-} -mcpu=neoverse-v2"
export CXXFLAGS="${CXXFLAGS:-} -mcpu=neoverse-v2"

# Build & package. `cargo pgrx package` produces a relocatable tree under
# target/release/<crate>-pg18/usr/pgsql-18/{lib,share/extension}/.
cargo pgrx package \
  --pg-config "/usr/pgsql-${PG_VERSION}/bin/pg_config" \
  --features pg18 \
  --no-default-features

PKG_ROOT=$(find target/release -maxdepth 2 -type d -name '*-pg18' -path '*release/*' | head -1)
test -n "$PKG_ROOT"

SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
OUT_TAR="target/ecaz-spire-aws-${SHA}.tar.gz"

(
  cd "${PKG_ROOT}/usr/pgsql-${PG_VERSION}"
  # Layout that bootstrap-node.sh expects:  lib/*.so  +  extension/*
  rm -rf "${WORK_DIR}/target/_aws_stage"
  mkdir -p "${WORK_DIR}/target/_aws_stage/lib" "${WORK_DIR}/target/_aws_stage/extension"
  cp -v lib/*.so "${WORK_DIR}/target/_aws_stage/lib/"
  cp -v share/extension/* "${WORK_DIR}/target/_aws_stage/extension/"
)
tar -C "${WORK_DIR}/target/_aws_stage" -czf "${WORK_DIR}/${OUT_TAR}" lib extension
sha256sum "${WORK_DIR}/${OUT_TAR}" | tee "${WORK_DIR}/${OUT_TAR}.sha256"

# Stable symlink-style copy for the Makefile to pick up.
cp -f "${WORK_DIR}/${OUT_TAR}" "${WORK_DIR}/target/ecaz-spire-aws-latest.tar.gz"
echo "TARBALL=${WORK_DIR}/${OUT_TAR}"
