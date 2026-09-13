#!/usr/bin/env bash
# Self-play Elo harness (delegates to the node-budget matcher).
# Usage: ./tools/tune.sh <old-binary> <new-binary> [rounds=40] [nodes-per-move=300000]
set -euo pipefail
cd "$(dirname "$0")/.."
LOG=tools/tune_log.csv
PY=python3
test -x "$1" || { echo "not executable: $1"; exit 1; }
test -x "$2" || { echo "not executable: $2"; exit 1; }
$PY tools/match.py "$1" "$2" "${3:-40}" "${4:-300000}" "$LOG"