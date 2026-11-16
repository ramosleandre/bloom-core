#!/bin/bash
# Runs the spike in an isolated, headless GNOME Shell: its own D-Bus session, runtime, config and
# data directories. Nothing is installed into the real session.
# Usage: dbus-run-session -- ./run.sh        (needs gnome-shell, zenity, python3)
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
W="$(mktemp -d)"; mkdir -p "$W/run" "$W/config" "$W/data/gnome-shell/extensions"; chmod 700 "$W/run"
cp -r "$HERE/sense-spike@bloom.local" "$W/data/gnome-shell/extensions/"
export XDG_RUNTIME_DIR="$W/run" XDG_DATA_HOME="$W/data" XDG_CONFIG_HOME="$W/config" GSETTINGS_BACKEND=keyfile
export SENSE_SPIKE_SOCK="$W/run/spike.sock"
unset WAYLAND_DISPLAY DISPLAY
gsettings set org.gnome.shell enabled-extensions "['sense-spike@bloom.local']"
python3 -I "$HERE/server.py" "$SENSE_SPIKE_SOCK" "$W/result.json" &
SRV=$!
gnome-shell --headless --wayland-display=spike-0 --virtual-monitor 1280x720 > "$W/shell.log" 2>&1 &
SH=$!
for _ in $(seq 1 40); do [ -S "$W/run/spike-0" ] && break; sleep 0.25; done
sleep 1
WAYLAND_DISPLAY=spike-0 GDK_BACKEND=wayland zenity --entry --title "Spike Fixture Window" \
  --text "fixture" --entry-text "select me" > "$W/zenity.out" 2>/dev/null &
ZN=$!
wait $SRV
kill $ZN $SH 2>/dev/null; sleep 0.5; kill -9 $SH 2>/dev/null
echo "fixture printed: [$(cat "$W/zenity.out" 2>/dev/null)]   (expected: pasted-by-spike)"
echo "results: $W/result.json   shell log: $W/shell.log"
