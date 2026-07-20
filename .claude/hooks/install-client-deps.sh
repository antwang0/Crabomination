#!/usr/bin/env bash
# SessionStart hook: install the system libraries the Bevy client crate needs
# to compile (wayland-sys / alsa-sys / libudev build scripts). Idempotent —
# skips the apt run once the pkg-config entries are present. Failures are
# non-fatal so engine/server work proceeds even if the mirror is unreachable.
set -u
need=0
for pc in wayland-client alsa libudev; do
  pkg-config --exists "$pc" 2>/dev/null || need=1
done
[ "$need" -eq 0 ] && exit 0
if command -v apt-get >/dev/null 2>&1; then
  apt-get update -qq >/dev/null 2>&1
  apt-get install -y libwayland-dev libasound2-dev libudev-dev libxkbcommon-dev \
    >/dev/null 2>&1 || echo "install-client-deps: apt install failed (client tests may not build)"
fi
exit 0
