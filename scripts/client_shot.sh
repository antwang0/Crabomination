#!/usr/bin/env bash
# One offscreen screenshot of crabomination_client, without a window on the
# desktop: a nested virtual KWin with its OWN D-Bus session and config dir.
# (On the desktop's session bus a nested KWin takes over kglobalaccel's `kwin`
# component and leaves Alt+Tab and friends switched off when it quits.)
#
#   scripts/client_shot.sh OUT.png [client args...]
#   env: W H (window, logical px; default 1920x1080), BIN (client binary,
#        default target/debug), TMO (seconds, default 90)
#
# Layout-harness flags: crabomination_client/src/layout_harness.rs
# (--layout-fixture 2|3|4, --stack-depth N, --hover-card NAME, --tokens,
#  --shortcut-help, --settings-open, ...). Needs kwin_wayland (KDE).
set -u
out=$(realpath -m "$1"); shift
repo=$(cd "$(dirname "$0")/.." && pwd)
W=${W:-1920}; H=${H:-1080}; TMO=${TMO:-90}
BIN=${BIN:-$repo/target/debug/crabomination_client}
state=${XDG_RUNTIME_DIR:-/tmp}/crab_shot; mkdir -p "$state/kwincfg"
sock=wayland-crab-$$
cd "$repo/crabomination_client"   # assets resolve from here
# KWin's title bar eats ~28 px, so the virtual screen is 40 px taller.
XDG_CONFIG_HOME=$state/kwincfg setsid dbus-run-session -- \
  kwin_wayland --virtual --width "$W" --height $((H + 40)) --socket "$sock" >"$state/kwin.log" 2>&1 &
for _ in $(seq 50); do [ -S "${XDG_RUNTIME_DIR}/$sock" ] && break; sleep 0.1; done
# DISPLAY unset, or winit takes XWayland on the real desktop.
env -u DISPLAY WAYLAND_DISPLAY="$sock" timeout "$TMO" "$BIN" --window "${W}x${H}" --screenshot "$out" "$@" \
  >"$state/client.log" 2>&1
rc=$?
# dbus-run-session's process group doesn't reach the KWin; kill it by pid.
for pid in $(ps -eo pid,args | grep "[k]win_wayland --virtual.*$sock" | grep -v "bash -c" | awk '{print $1}'); do
  kill "$pid"
done
[ -f "$out" ] || { echo "no screenshot (client exit $rc; log: $state/client.log)" >&2; exit 1; }
echo "$out"
