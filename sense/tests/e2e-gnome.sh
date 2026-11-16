#!/bin/bash
# End to end on GNOME: the daemon, the real Shell extension and the CLI, in an isolated headless
# GNOME Shell (own D-Bus session, runtime, config and data directories). Nothing is installed
# into the real session.
#
#   cargo build && dbus-run-session -- tests/e2e-gnome.sh
#
# Needs gnome-shell and zenity.
set -u
HERE="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${SENSE_BIN:-$HERE/target/debug}"
W="$(mktemp -d)"; mkdir -p "$W/run" "$W/config" "$W/data/gnome-shell/extensions"; chmod 700 "$W/run"
cp -r "$HERE/ext/gnome-shell/sense@bloom.local" "$HERE/tests/fixtures/sense-e2e-helper@bloom.local" "$W/data/gnome-shell/extensions/"
export XDG_RUNTIME_DIR="$W/run" XDG_DATA_HOME="$W/data" XDG_CONFIG_HOME="$W/config" GSETTINGS_BACKEND=keyfile
unset WAYLAND_DISPLAY DISPLAY SENSE_SOCKET
export XDG_CURRENT_DESKTOP=GNOME XDG_SESSION_TYPE=wayland
gsettings set org.gnome.shell enabled-extensions "['sense@bloom.local', 'sense-e2e-helper@bloom.local']"
LEVEL="${1:-on_request}"

gnome-shell --headless --wayland-display=sense-e2e --virtual-monitor 1280x720 > "$W/shell.log" 2>&1 &
SHELL_PID=$!
for _ in $(seq 1 40); do [ -S "$W/run/sense-e2e" ] && break; sleep 0.25; done
sleep 1
# A bare D-Bus session has no systemd to start the accessibility registry: start it by hand.
REGISTRYD="$(command -v at-spi2-registryd || ls /usr/libexec/at-spi2-registryd /usr/lib/at-spi2-core/at-spi2-registryd 2>/dev/null | head -1)"
gdbus call --session --dest org.a11y.Bus --object-path /org/a11y/bus --method org.a11y.Bus.GetAddress > /dev/null 2>&1
[ -n "$REGISTRYD" ] && "$REGISTRYD" > "$W/registryd.log" 2>&1 &
REGISTRY_PID=$!
sleep 0.5
"$BIN/sensed" --dev-no-indicator --level "$LEVEL" 2> "$W/sensed.log" &
SENSED=$!
WAYLAND_DISPLAY=sense-e2e GDK_BACKEND=wayland zenity --entry --title "Sense Fixture Window" \
  --text "fixture" --entry-text "select me" > /dev/null 2>&1 &
FIXTURE=$!
sleep 4

fail=0
check() { if echo "$2" | grep -qF -- "$3"; then echo "  ok    $1"; else echo "  FAIL  $1 (expected: $3)"; fail=1; fi; }
refuse() { if echo "$2" | grep -qF -- "$3"; then echo "  FAIL  $1 (found: $3)"; fail=1; else echo "  ok    $1"; fi; }

echo "== capabilities"
CAPS="$("$BIN/sense" caps)"
check "the extension registered as linux.gnome" "$CAPS" '"linux.gnome"'

echo "== capture, as a card (level: $LEVEL)"
CARD="$("$BIN/sense" capture --want session,focus,selection,document,clipboard,pointer --card)"
echo "$CARD" | sed 's/^/    /'
check "window title read through the whole chain" "$CARD" 'window: "Sense Fixture Window"'
check "application identified" "$CARD" 'app: zenity'
check "platform reported" "$CARD" 'linux/gnome'
# Informative, not asserted: in a headless Shell the GTK fixture does not report itself as focused
# over AT-SPI, so linux.atspi rightly finds no focused element. See spec/14-open-questions.md, S8.
info() { if echo "$2" | grep -qF -- "$3"; then echo "  ok    $1"; else echo "  info  $1: not observed in this environment"; fi; }
info "focused element described through accessibility" "$CARD" 'focus: text field, editable'
info "exact selection read from the focused field" "$CARD" 'selection (exact, 9 chars): see <selection>'
check "what has no source is reported, not invented" "$CARD" 'document (no source in this session)'

echo "== capture, as JSON: timings"
JSON="$("$BIN/sense" capture --want focus,pointer,selection)"
echo "$JSON" | grep -A3 '"timings_ms"' | sed 's/^/    /'
check "pointer read from the compositor" "$JSON" '"via": "linux.gnome"'
check "accessibility source took part in the capture" "$JSON" '"linux.atspi"'

echo "== access ledger"
LOG="$("$BIN/sense" log)"
check "the reads are in the ledger" "$LOG" '"client": "sense-cli"'
refuse "the ledger holds no title" "$LOG" 'Sense Fixture Window'
refuse "the ledger holds no selected text" "$LOG" 'select me'

if [ "$LEVEL" = on_request ]; then
  echo "== nothing retained at the on-request level"
  TL="$("$BIN/sense" timeline)"
  refuse "no focus events kept" "$TL" 'focus.changed'
fi

kill $FIXTURE $SHELL_PID $SENSED $REGISTRY_PID 2>/dev/null; sleep 0.5; kill -9 $SHELL_PID 2>/dev/null
echo "logs: $W"
exit $fail
