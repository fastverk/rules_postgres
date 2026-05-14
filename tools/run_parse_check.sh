#!/usr/bin/env bash
# run_parse_check.sh — sh_test wrapper that forwards args to parse_check.
# Usage: run_parse_check.sh <parse_check-binary> <sql-file>
set -euo pipefail
exec "$1" "$2"
