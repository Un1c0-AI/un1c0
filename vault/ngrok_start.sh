#!/usr/bin/env bash
set -euo pipefail

# Start ngrok to expose the local admin service so GH Actions can reach it for PoC testing.
# Requires `ngrok` installed and authtoken configured.

if ! command -v ngrok >/dev/null 2>&1; then
  echo "Install ngrok and run 'ngrok authtoken <token>' first." >&2
  exit 2
fi

echo "Starting ngrok tunnel to http://localhost:5000"
ngrok http 5000
