#!/bin/bash
# Iterate strip-bad-default + add_default_v2 until no E0063 or E0436 errors remain.
set -e
for i in {1..10}; do
    python scripts/strip_bad_default.py
    sleep 1
    python scripts/add_default_v2.py
    sleep 1
    n=$(cargo check 2>&1 | grep -c 'E0063\|E0436' || true)
    echo "=== iteration $i: $n E0063/E0436 errors ==="
    if [ "$n" -eq 0 ]; then
        echo "Done!"
        break
    fi
done
