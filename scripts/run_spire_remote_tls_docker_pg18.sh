#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
OPENSSL="${OPENSSL:-openssl}"
DOCKER_IMAGE="${DOCKER_IMAGE:-postgres:latest}"
REMOTE_PORT="${REMOTE_PORT:-39418}"
COORD_PORT="${COORD_PORT:-39419}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_remote_tls_docker_pg18.sh [options]

Options:
  --artifact-dir DIR  Store smoke, Docker, and PostgreSQL logs in DIR.
  --coord-port PORT   Coordinator PostgreSQL port. Default: 39419.
  --docker-image IMG  TLS remote PostgreSQL image. Default: postgres:latest.
  --log-dir DIR       Store PostgreSQL logs in DIR.
  --pgbin DIR         PostgreSQL bin directory for coordinator. Default: $PGBIN.
  --remote-port PORT  Docker-published remote PostgreSQL port. Default: 39418.
  --run-dir DIR       Run directory. Default: target/spire-remote-tls-docker-pg18-$RUN_ID.
  --run-id ID         Run id used in the default run directory and container name.
  --skip-install      Skip cargo pgrx install.
  --smoke-log FILE    Tee smoke output to FILE.
  -h, --help          Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --coord-port)
      COORD_PORT="$2"
      shift 2
      ;;
    --docker-image)
      DOCKER_IMAGE="$2"
      shift 2
      ;;
    --log-dir)
      LOG_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --pgbin)
      PGBIN="$2"
      PG_CTL="$PGBIN/pg_ctl"
      PSQL="$PGBIN/psql"
      shift 2
      ;;
    --remote-port)
      REMOTE_PORT="$2"
      shift 2
      ;;
    --run-dir)
      RUN_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --skip-install)
      ECAZ_SKIP_INSTALL=1
      shift
      ;;
    --smoke-log)
      SMOKE_LOG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-remote-tls-docker-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/remote-tls-docker-success.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
COORD_DATA="$RUN_DIR/coord"
SOCKET_KEY="$(printf '%s' "$RUN_DIR" | cksum | awk '{print $1}')"
SOCKET_DIR="${SOCKET_DIR:-$ROOT_DIR/target/s-$SOCKET_KEY}"
CERT_DIR="$RUN_DIR/certs"
CA_CERT="$CERT_DIR/ca.crt"
CONTAINER_NAME="ecaz-spire-tls-${RUN_ID//[^A-Za-z0-9_.-]/-}"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_SMOKE_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_SMOKE_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR" "$CERT_DIR"
: > "$LOG_DIR/remote-postgres.log"
: > "$LOG_DIR/coord-postgres.log"

cleanup() {
  "$PG_CTL" -D "$COORD_DATA" -m fast stop >/dev/null 2>&1 || true
  if [[ "${ECAZ_TLS_REMOTE_CONTAINER_STARTED:-0}" == "1" ]]; then
    docker logs "$CONTAINER_NAME" > "$LOG_DIR/remote-postgres.log" 2>&1 || true
    docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "remote_port=$REMOTE_PORT"
echo "coord_port=$COORD_PORT"
echo "docker_image=$DOCKER_IMAGE"
echo "container_name=$CONTAINER_NAME"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$OPENSSL" req -new -x509 -days 2 -nodes \
  -subj "/CN=ecaz-spire-local-ca" \
  -keyout "$CERT_DIR/ca.key" \
  -out "$CA_CERT" >/dev/null 2>&1
"$OPENSSL" req -new -nodes \
  -subj "/CN=localhost" \
  -keyout "$CERT_DIR/server.key" \
  -out "$CERT_DIR/server.csr" >/dev/null 2>&1
cat > "$CERT_DIR/server.ext" <<'EOF'
subjectAltName=DNS:localhost
extendedKeyUsage=serverAuth
EOF
"$OPENSSL" x509 -req -days 2 \
  -in "$CERT_DIR/server.csr" \
  -CA "$CA_CERT" \
  -CAkey "$CERT_DIR/ca.key" \
  -CAcreateserial \
  -out "$CERT_DIR/server.crt" \
  -extfile "$CERT_DIR/server.ext" >/dev/null 2>&1
cat > "$CERT_DIR/pg_hba.conf" <<'HBA'
local all all trust
hostssl all all all trust
hostnossl all all all reject
HBA

docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
docker run -d \
  --name "$CONTAINER_NAME" \
  -e POSTGRES_HOST_AUTH_METHOD=trust \
  -p "127.0.0.1:$REMOTE_PORT:5432" \
  -v "$CERT_DIR:/certs:ro" \
  --entrypoint bash \
  "$DOCKER_IMAGE" \
  -c "set -euo pipefail; \
       cp /certs/server.crt /var/lib/postgresql/server.crt; \
       cp /certs/server.key /var/lib/postgresql/server.key; \
       chown postgres:postgres /var/lib/postgresql/server.crt /var/lib/postgresql/server.key; \
       chmod 600 /var/lib/postgresql/server.key; \
       exec docker-entrypoint.sh postgres \
         -c ssl=on \
         -c ssl_cert_file=/var/lib/postgresql/server.crt \
         -c ssl_key_file=/var/lib/postgresql/server.key \
         -c hba_file=/certs/pg_hba.conf" >/dev/null
ECAZ_TLS_REMOTE_CONTAINER_STARTED=1

for _ in {1..60}; do
  if docker exec "$CONTAINER_NAME" pg_isready -U postgres >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
if ! docker exec "$CONTAINER_NAME" pg_isready -U postgres >/dev/null 2>&1; then
  docker logs "$CONTAINER_NAME" > "$LOG_DIR/remote-postgres.log" 2>&1 || true
  echo "Docker TLS remote did not become ready" >&2
  exit 6
fi

REQUIRE_CONNINFO="host=localhost port=$REMOTE_PORT dbname=postgres user=postgres sslmode=require target_session_attrs=read-write"
VERIFY_FULL_CONNINFO="host=localhost port=$REMOTE_PORT dbname=postgres user=postgres sslmode=verify-full sslrootcert=$CA_CERT target_session_attrs=read-write"
DISABLE_CONNINFO="host=localhost port=$REMOTE_PORT dbname=postgres user=postgres sslmode=disable"
BAD_HOST_CONNINFO="host=127.0.0.1 port=$REMOTE_PORT dbname=postgres user=postgres sslmode=verify-full sslrootcert=$CA_CERT"
sql_literal() {
  printf "%s" "$1" | sed "s/'/''/g"
}
REQUIRE_CONNINFO_SQL="$(sql_literal "$REQUIRE_CONNINFO")"
VERIFY_FULL_CONNINFO_SQL="$(sql_literal "$VERIFY_FULL_CONNINFO")"
DISABLE_CONNINFO_SQL="$(sql_literal "$DISABLE_CONNINFO")"
BAD_HOST_CONNINFO_SQL="$(sql_literal "$BAD_HOST_CONNINFO")"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_TLS_REQUIRE="$REQUIRE_CONNINFO"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_TLS_VERIFY_FULL="$VERIFY_FULL_CONNINFO"

"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

require_probe="$("${coord_psql[@]}" -At -F ',' -c "SELECT connection_status, ssl::text, tls_version FROM tests.ec_spire_test_remote_conninfo_tls_probe('$REQUIRE_CONNINFO_SQL')")"
verify_full_probe="$("${coord_psql[@]}" -At -F ',' -c "SELECT connection_status, ssl::text, tls_version FROM tests.ec_spire_test_remote_conninfo_tls_probe('$VERIFY_FULL_CONNINFO_SQL')")"
disable_probe="$("${coord_psql[@]}" -At -F ',' -c "SELECT connection_status, ssl::text FROM tests.ec_spire_test_remote_conninfo_tls_probe('$DISABLE_CONNINFO_SQL')")"
bad_host_probe="$("${coord_psql[@]}" -At -F ',' -c "SELECT connection_status, ssl::text FROM tests.ec_spire_test_remote_conninfo_tls_probe('$BAD_HOST_CONNINFO_SQL')")"

echo "require_probe=$require_probe"
echo "verify_full_probe=$verify_full_probe"
echo "disable_probe=$disable_probe"
echo "bad_host_probe=$bad_host_probe"

[[ "$require_probe" == connected,true,* || "$require_probe" == connected,t,* ]]
[[ "$verify_full_probe" == connected,true,* || "$verify_full_probe" == connected,t,* ]]
[[ "$disable_probe" == "connect_failed,false" || "$disable_probe" == "connect_failed,f" ]]
[[ "$bad_host_probe" == "connect_failed,false" || "$bad_host_probe" == "connect_failed,f" ]]

require_transport="$("${coord_psql[@]}" -At -F ',' -c "
SELECT node_id::text || ',' || status || ',' || failure_category || ',' || row_count::text
  FROM tests.ec_spire_test_production_transport_probe(
       ARRAY[2]::integer[],
       ARRAY['spire/remote/tls_require']::text[],
       0
  )
")"
verify_full_transport="$("${coord_psql[@]}" -At -F ',' -c "
SELECT node_id::text || ',' || status || ',' || failure_category || ',' || row_count::text
  FROM tests.ec_spire_test_production_transport_probe(
       ARRAY[3]::integer[],
       ARRAY['spire/remote/tls_verify_full']::text[],
       0
  )
")"

echo "require_transport=$require_transport"
echo "verify_full_transport=$verify_full_transport"

[[ "$require_transport" == 2,ready,none,* ]]
[[ "$verify_full_transport" == 3,ready,none,* ]]
[[ "${require_transport##*,}" -gt 0 ]]
[[ "${verify_full_transport##*,}" -gt 0 ]]

docker logs "$CONTAINER_NAME" > "$LOG_DIR/remote-postgres.log" 2>&1 || true
echo "SPIRE remote TLS Docker PG18 probe passed"
