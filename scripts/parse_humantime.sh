#!/usr/bin/env bash
# Convert a humantime-style duration (e.g. "5", "300s", "1h", "24h", "1d")
# to integer seconds. Used by `make soak DURATION=…` so the Make wrapper
# stays one line while accepting the same syntax the Rust soak harness'
# docs reference.
set -euo pipefail

input="${1:-}"
if [ -z "$input" ]; then
  echo "usage: $0 <duration>" >&2
  exit 2
fi

# Strip a single trailing unit suffix and convert. Bash regex is enough
# for the small set of units we accept.
if [[ "$input" =~ ^([0-9]+)$ ]]; then
  echo "${BASH_REMATCH[1]}"
elif [[ "$input" =~ ^([0-9]+)s$ ]]; then
  echo "${BASH_REMATCH[1]}"
elif [[ "$input" =~ ^([0-9]+)m$ ]]; then
  echo "$(( ${BASH_REMATCH[1]} * 60 ))"
elif [[ "$input" =~ ^([0-9]+)h$ ]]; then
  echo "$(( ${BASH_REMATCH[1]} * 3600 ))"
elif [[ "$input" =~ ^([0-9]+)d$ ]]; then
  echo "$(( ${BASH_REMATCH[1]} * 86400 ))"
else
  echo "unsupported duration format: $input (use NNN, NNNs, NNNm, NNNh, NNNd)" >&2
  exit 2
fi
