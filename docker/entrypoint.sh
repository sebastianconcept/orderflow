#!/bin/sh
set -euo pipefail

# First CLI arg (compose `command` / Kamal `cmd`) overrides ORDERFLOW_BIN.
if [ "$#" -gt 0 ]; then
  binary_name="$1"
else
  binary_name="${ORDERFLOW_BIN:-engine-core}"
fi

binary_path="/app/bin/${binary_name}"

if [ ! -x "${binary_path}" ]; then
  echo "orderflow: binary not found: ${binary_path}" >&2
  exit 1
fi

echo "orderflow: starting ${binary_name} (ORDERFLOW_ENV=${ORDERFLOW_ENV:-unknown})"
"${binary_path}"

# Placeholder binaries exit immediately; keep the container up for compose until services are long-running.
if [ "${ORDERFLOW_HOLD:-0}" = "1" ]; then
  echo "orderflow: ${binary_name} exited (placeholder); ORDERFLOW_HOLD=1 — sleeping"
  exec sleep infinity
fi
