# Spike: a GNOME Shell extension as a context source

Throwaway code. It answers part of the open questions in
[`../../spec/14-open-questions.md`](../../spec/14-open-questions.md). Do not build on it; read it,
then write the real extension from the specs.

## What it is

- `sense-spike@bloom.local/`: a 110-line extension that **connects out** to a Unix socket and
  answers a few commands: focused window, pointer, read PRIMARY or CLIPBOARD, list the selection
  owners' formats, send key codes through a virtual keyboard, set the clipboard and paste.
- `server.py`: the other end. It drives one scripted session and writes the answers with timings.
- `run.sh`: starts a **headless GNOME Shell in an isolated session** (its own D-Bus, runtime,
  configuration and data directories), enables the extension there, opens a small fixture window
  with a text field, and runs the script. Nothing is installed into the real session.

```
dbus-run-session -- ./run.sh
```

## Results (one machine, GNOME Shell 50.6, headless, 16 November 2026)

One environment and one toolkit (a GTK 4 text field). These results show that the mechanisms
work; they do not measure reliability across applications.

| Question | Result |
|---|---|
| S1 · Can the extension connect out to a Unix socket with asynchronous I/O, and reconnect? | **Yes.** Request and reply in about 0.5 ms (median), worst 3 ms. After the server closed the connection, the extension was back in 0.3 s (its retry delay). |
| Focused window from inside the Shell | **Yes.** Title, class and process id of the fixture window; about 0.3 ms inside the Shell. |
| Does a window activated by the extension get keyboard focus? | **Yes** for this GTK window: keys sent afterwards reached its text field. |
| S3 · Paste by virtual keyboard | **Yes, once.** The extension set the clipboard, sent Ctrl+V 50 ms later, then Enter; the fixture printed exactly the pasted text, which had replaced the selected text. |
| Read another application's CLIPBOARD | **Yes.** After a Ctrl+C sent to the fixture, `St.Clipboard` returned the copied text in about 1 ms. |
| S2 · Read the PRIMARY selection | **No, and this changes the design.** With GNOME's default settings there was no PRIMARY owner at all while text was selected. Cause: since GNOME 50, primary paste is disabled by default (`org.gnome.desktop.interface gtk-enable-primary-paste` is `false`), and GTK applications then do not publish a primary selection. With the setting turned on, an owner appeared but `St.Clipboard.get_text(PRIMARY)` returned an empty string. |

Not covered here: S4 (macOS), S5 (capture latency end to end), S6 (browser tab matching), S7 with
a WebKitGTK window, S3 at scale and after focus was taken by our own window, and anything on
GNOME 46 to 49.

## Consequence

On GNOME 50 and later, PRIMARY cannot be the way to get the selection. The specs were changed:
the selection on GNOME comes from the browser extension, then accessibility, then a **copy-based
read** (send the copy shortcut to the target, read the clipboard, restore it), which this spike
showed working. See `spec/platform-linux.md` and `spec/12-acting.md`.

Source for the default change: GNOME's `gsettings-desktop-schemas` merge request "Disable
primary-paste by default", confirmed by Debian's changelog for the package and by a Fedora bug
closed as intentional.
