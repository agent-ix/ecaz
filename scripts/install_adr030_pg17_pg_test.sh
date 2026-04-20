#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/install_adr030_pg17_pg_test.sh \
      [--pgrx-home /tmp/tqvector_pgrx_home] \
      [--pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config]

Notes:
  - Installs the pg17 `pg_test` build used by ADR-030 scratch diagnostics.
  - Forces the pg17 `pg_config` path so `cargo pgrx install` does not fall back
    to the system PostgreSQL build.
EOF
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

pgrx_home="${PGRX_HOME:-/tmp/tqvector_pgrx_home}"
pg_config="/home/peter/.pgrx/17.9/pgrx-install/bin/pg_config"
release_artifact="${repo_root}/target/release/libecaz.so"

artifact_sha256() {
    sha256sum "$1" | awk '{print $1}'
}

artifact_build_id() {
    readelf -n "$1" 2>/dev/null | awk '/Build ID:/ { print $3; exit }'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --pgrx-home)
            pgrx_home="$2"
            shift 2
            ;;
        --pg-config)
            pg_config="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

printf '[install] repo=%s\n' "${repo_root}"
printf '[install] pgrx_home=%s\n' "${pgrx_home}"
printf '[install] pg_config=%s\n' "${pg_config}"

cd "${repo_root}"
PGRX_HOME="${pgrx_home}" cargo pgrx install \
    --pg-config "${pg_config}" \
    --release \
    --features "pg17 pg_test" \
    --no-default-features

pkglibdir="$("${pg_config}" --pkglibdir)"
installed_backend="${pkglibdir}/ecaz.so"

if [[ ! -f "${release_artifact}" ]]; then
    echo "[install] expected release artifact missing: ${release_artifact}" >&2
    exit 1
fi

if [[ ! -f "${installed_backend}" ]]; then
    echo "[install] installed backend missing: ${installed_backend}" >&2
    exit 1
fi

if ! cmp -s "${release_artifact}" "${installed_backend}"; then
    echo "[install] backend .so mismatch after install" >&2
    echo "[install] built=${release_artifact}" >&2
    echo "[install] installed=${installed_backend}" >&2
    echo "[install] built_sha256=$(artifact_sha256 "${release_artifact}")" >&2
    echo "[install] installed_sha256=$(artifact_sha256 "${installed_backend}")" >&2

    built_build_id="$(artifact_build_id "${release_artifact}")"
    installed_build_id="$(artifact_build_id "${installed_backend}")"
    if [[ -n "${built_build_id}" || -n "${installed_build_id}" ]]; then
        echo "[install] built_build_id=${built_build_id:-unknown}" >&2
        echo "[install] installed_build_id=${installed_build_id:-unknown}" >&2
    fi

    exit 1
fi

echo "[install] backend .so assertion passed"
echo "[install] installed_backend=${installed_backend}"
echo "[install] sha256=$(artifact_sha256 "${installed_backend}")"
