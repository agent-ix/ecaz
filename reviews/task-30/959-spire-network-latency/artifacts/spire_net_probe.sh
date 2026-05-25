#!/usr/bin/env bash
# Tier-1 inter-node network probe: TCP connect RTT + libpq SELECT 1.
# Runs on the coordinator, targets the remote private IP.
set -uo pipefail
R="10.42.1.58"
P=5432

echo "=== wait for remote PG (max 1500s) ==="
OPEN=0
for i in $(seq 1 300); do
  if timeout 3 bash -c "</dev/tcp/$R/$P" 2>/dev/null; then
    echo "PG_PORT_OPEN after $((i*5))s"
    OPEN=1
    break
  fi
  sleep 5
done
[ "$OPEN" = "1" ] || { echo "PG_NEVER_CAME_UP"; exit 0; }

echo "=== tcp connect rtt x100 ==="
python3 - <<'PY'
import socket, time
R="10.42.1.58"; P=5432; ts=[]
for _ in range(100):
    s=socket.socket(); t0=time.perf_counter()
    try:
        s.connect((R,P)); ts.append((time.perf_counter()-t0)*1000)
    except Exception as e:
        print("ERR", e)
    finally:
        s.close()
if ts:
    ts.sort(); n=len(ts)
    print(f"tcp_connect_ms n={n} min={ts[0]:.3f} p50={ts[n//2]:.3f} p95={ts[min(int(n*0.95),n-1)]:.3f} p99={ts[min(int(n*0.99),n-1)]:.3f} max={ts[-1]:.3f} mean={sum(ts)/n:.3f}")
PY

echo "=== libpq connect + SELECT 1 x100 ==="
PSQL=$(command -v psql || true)
if [ -z "$PSQL" ] || [ ! -x "$PSQL" ]; then
  for c in /usr/bin/psql /usr/pgsql-18/bin/psql /usr/lib/postgresql/18/bin/psql; do [ -x "$c" ] && PSQL="$c"; done
fi
echo "psql=$PSQL"
python3 - <<PY
import subprocess, time
R="10.42.1.58"; P=5432; PSQL="$PSQL"; ts=[]; errs=0
for _ in range(100):
    t0=time.perf_counter()
    r=subprocess.run([PSQL,"-h",R,"-p",str(P),"-U","ecaz_coord","-d","postgres","-tAc","SELECT 1"],
                     capture_output=True, text=True, timeout=10)
    dt=(time.perf_counter()-t0)*1000
    if r.returncode==0 and r.stdout.strip()=="1":
        ts.append(dt)
    else:
        errs+=1
        if errs<=3: print("ERR", r.returncode, r.stderr.strip()[:200])
if ts:
    ts.sort(); n=len(ts)
    print(f"libpq_select1_ms n={n} errs={errs} min={ts[0]:.3f} p50={ts[n//2]:.3f} p95={ts[min(int(n*0.95),n-1)]:.3f} p99={ts[min(int(n*0.99),n-1)]:.3f} max={ts[-1]:.3f} mean={sum(ts)/n:.3f}")
else:
    print("libpq_select1 ALL FAILED errs=", errs)
PY
echo "=== done ==="
