#!/usr/bin/env bash
set -euo pipefail

install_path="${HOME}/.local/bin/hyprosd"
daemon_path="${HOME}/.local/bin/hyprosd-daemon"

if [ -f "$install_path" ]; then
  rm -f "$install_path"
  echo "Removed ${install_path}"
else
  echo "No installed binary found at ${install_path}"
fi

if [ -f "$daemon_path" ]; then
  rm -f "$daemon_path"
  echo "Removed ${daemon_path}"
else
  echo "No daemon launcher found at ${daemon_path}"
fi

if command -v pkill >/dev/null 2>&1; then
  pkill hyprosd || true
fi
