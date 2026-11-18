#!/bin/sh
# Everything a change must pass before it is committed (ARCHITECTURE.md, "The rules every change follows"):
# the tests, the build for the browser with no warnings, and what tools/limits.py checks of the rules.
set -e
cd "$(dirname "$0")"

echo "== tests"
cargo test --quiet

echo "== build for the browser"
built=$(./build-web.sh 2>&1) || { echo "$built"; exit 1; }
if echo "$built" | grep -q "^warning"; then
    echo "$built"
    echo "the build has warnings"
    exit 1
fi

echo "== sizes and names"
python3 tools/limits.py

echo "all checks passed"
