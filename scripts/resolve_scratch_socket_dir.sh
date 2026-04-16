#!/usr/bin/env bash
set -euo pipefail

port="${TQV_PG_PORT:-${PGPORT:-28817}}"
preferred_socket_dir="/tmp/tqvector_pgrx_home"
fallback_socket_dir="${HOME}/.pgrx"
socket_name=".s.PGSQL.${port}"

socket_exists() {
    local path="$1"
    if [[ -S "${path}" ]]; then
        return 0
    fi
    if [[ "${TQV_SCRATCH_TEST_ACCEPT_FILES:-0}" == "1" && -e "${path}" ]]; then
        return 0
    fi
    return 1
}

if [[ -n "${TQV_PG_SOCKET_DIR:-}" ]]; then
    printf '%s\n' "${TQV_PG_SOCKET_DIR}"
    exit 0
fi

if [[ -n "${PGHOST:-}" ]]; then
    printf '%s\n' "${PGHOST}"
    exit 0
fi

if socket_exists "${preferred_socket_dir}/${socket_name}"; then
    printf '%s\n' "${preferred_socket_dir}"
    exit 0
fi

if socket_exists "${fallback_socket_dir}/${socket_name}"; then
    printf 'scratch wrapper refusing to fall back to %s; set TQV_PG_SOCKET_DIR or PGHOST explicitly if that cluster is intended\n' "${fallback_socket_dir}" >&2
    exit 1
fi

printf 'scratch wrapper expected socket at %s/%s; set TQV_PG_SOCKET_DIR or PGHOST explicitly to target a different cluster\n' "${preferred_socket_dir}" "${socket_name}" >&2
exit 1
