#!/usr/bin/env bash
# Inverts the exit code of parse_check — passes iff parse_check FAILS.
# Used for negative tests (bad SQL must be rejected).
set -uo pipefail

if "$1" "$2"; then
  echo "FAIL: parse_check accepted $2, but it should have been rejected" >&2
  exit 1
fi
exit 0
