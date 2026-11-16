# 11 · Runtime: processes, lifecycle, indicator, configuration

How the pieces run on a real machine. This file settles questions the other specs left open: who
starts the daemon, who draws the indicator, where configuration lives, and what happens when a
piece is missing or crashes.

## Processes

| Process | What it is | Started by | Lives |
|---|---|---|---|
| `sensed` | the daemon: sources, core, API, the acting service (`12`) | the first-party client (the Bloom shell) at its own start; optionally at login (below) | until the client quits, or the session ends |
| `sense-tray` | the indicator: a tiny native tray or menu-bar program | `sensed`, always | as long as `sensed` |
| `sense-nmh` | browser bridge | the browser, when the extension connects | as long as the browser keeps the port open |
| `sense-mcp` | MCP server for other assistants | the assistant that uses it | per assistant session |
| extensions | GNOME Shell extension, KWin script, editor extension, shell hooks | their host | as long as their host |

One `sensed` per user session. A second instance finds the socket in use and exits.

## Who draws the indicator

The daemon does, through `sense-tray`, not the client. Reasons:

- "A visible indicator whenever a source runs" must hold even if the client's interface is closed,
  crashed, or is a third party's.
- A client cannot then ship without it or hide it.

`sense-tray` is a separate small binary (so the daemon stays free of GUI toolkits) that connects to
the daemon like any reader, with the `admin` scope granted by path. If `sense-tray` cannot start or
exits, `sensed` stops every source within 5 seconds and reports `source.state: stopped` with code
`no_indicator`. Observation without an indicator is not a state the system can be in.

What it shows:

- an icon with three states: paused, on request, session awareness;
- a brief change of the icon on every `capture` or `content.read`;
- a menu: the last reads (client, application, kinds: from the access ledger), pause for 15 minutes
  / 1 hour / until resumed, the observation level, "what Sense can see" (opens the inspector),
  settings, quit.

Per platform: `NSStatusItem` on macOS; a notification-area icon on Windows; the
StatusNotifierItem D-Bus protocol on Linux. `sense-tray` checks that a StatusNotifierWatcher is
present on the session bus before claiming the indicator is visible. Stock GNOME has none (Ubuntu
adds one by default, Fedora does not), so on GNOME **our own Shell extension draws the indicator
in the top bar** and takes over from `sense-tray`; the same 5-second rule applies to the
extension's connection.
On a GNOME session without our extension there is no window-level observation anyway, and the
family-independent sources refuse to start until some indicator is present.

Consent screens and settings are drawn by the first-party client (it has a real UI toolkit), using
`capabilities.get` and `privacy.*`. The daemon enforces; the client only presents. The one thing
the daemon will not accept from an ordinary client is raising the level (below).

## Start-up sequence

1. Client launches `sensed` with its own pid (`--parent-pid`) and the paths of the binaries that
   may hold the `admin` scope.
2. `sensed` creates the socket, loads configuration, starts the indicator.
3. If no consent is recorded: level is `Paused`, no source starts, `capabilities` says
   `needs_consent`. The client shows onboarding and calls `privacy.set`.
4. Otherwise: build `SessionEnv`, probe sources, start those the level and the configuration allow.
5. Extensions connect when their hosts are running; each connection re-publishes capabilities.

Without a login item, the daemon dies with its parent (it watches `--parent-pid`). That is the
default and all the minimum slice needs.

**At-login mode** (required only for observation level 2 to be useful, since history needs the
daemon to run before the client is opened): a LaunchAgent via `SMAppService` on macOS, an XDG
autostart entry or systemd user unit on Linux, a per-user Run entry on Windows. Enabling level 2
offers it; it is never installed silently. On macOS this mode has an unverified permission
question (`platform-macos.md`).

## Failure behaviour

| Event | Behaviour |
|---|---|
| A source panics or errors | That source stops, `source.state` is emitted, capabilities update. The daemon continues. A source that fails three times in ten minutes stays stopped until the next session. |
| An application hangs during an accessibility read | The read times out; the application is put on a 30-second cool-down for that source. |
| An extension disconnects | Its signals become `Unavailable(NoSource)`. Reconnection restores them. |
| The daemon crashes | State is lost by design. The client detects the closed socket, restarts it once, and shows "context unavailable" in the meantime. A crash never leaves a source running: sources are threads of the daemon or extensions that stop sending when the socket closes. |
| The client crashes | With `--parent-pid`, the daemon exits. In at-login mode it keeps running, at its level, with its indicator. |
| Version mismatch between daemon and an extension | The handshake carries protocol versions. A source with an older minor version works with fewer signals; a different major version is refused with a `needs_install` capability telling the user to update it. |

## Configuration and data on disk

| What | Linux | macOS | Windows |
|---|---|---|---|
| Config (`config.toml`): level, exclusions, redaction switches, ledger option | `$XDG_CONFIG_HOME/sense/` | `~/Library/Application Support/Sense/` | `%APPDATA%\Sense\` |
| Grants (`grants.toml`): reader scopes and per-application approvals | same | same | same |
| Source tokens (`tokens/`, mode 0600) | `$XDG_STATE_HOME/sense/` | same folder | same folder |
| Socket | `$XDG_RUNTIME_DIR/sense/sense.sock` | `~/Library/Application Support/Sense/sense.sock` | `\\.\pipe\sense-<SID>` |
| Access ledger, only if enabled | `$XDG_STATE_HOME/sense/ledger.jsonl` | same folder | same folder |
| Logs (no content) | `$XDG_STATE_HOME/sense/log/` | `~/Library/Logs/Sense/` | `%LOCALAPPDATA%\Sense\log\` |

Rules:

- All files are user-only. Nothing else is ever written: no state, no timeline, no content.
- `config.toml` and `grants.toml` are plain, documented, hand-editable. The daemon watches them.
- **Raising the observation level** is accepted only over a connection whose executable is listed
  as `admin`, and is recorded with a timestamp in `config.toml` (`consent.level2_at`). Editing the
  file by hand works too: the user owns their machine.
- `sense reset` deletes every file above and unregisters extensions and manifests; the uninstaller
  calls it.

```toml
# config.toml (shape)
[consent]
level = "on_request"            # paused | on_request | session_awareness
accepted_at = "2026-11-16T10:00:00Z"

[exclude]
apps   = ["signal", "org.keepassxc.KeePassXC"]      # canonical names or raw ids
titles = ["*Private*"]
urls   = ["*://*.mybank.example/*"]
categories = ["password_manager"]

[redaction]
enabled = true
except_apps = ["terminal"]

[ledger]
persist = false
retention_days = 30

[timeline]
minutes = 60
clear_on_lock = true
```

## Installing the pieces

| Piece | macOS | Windows | Linux |
|---|---|---|---|
| `sensed`, `sense-tray`, `sense`, `sense-nmh` | inside the client's application bundle, same signature | inside the client's installer; a sparse package gives identity for the WinRT sources | inside the client's native package (deb, rpm, Arch) or tarball; never a Flatpak or Snap |
| Browser extension | Chrome Web Store, Firefox Add-ons | same | same |
| Native-messaging manifest | written by the client's installer or first run; removed by `sense reset` | registry key | JSON file in each browser's directory |
| GNOME Shell extension | n/a | n/a | extensions.gnome.org, or shipped in the native package; needs a re-login on Wayland |
| KWin script | n/a | n/a | loaded at run time by the daemon; nothing installed |
| Shell hooks, editor extension | optional, user-installed from the client's settings | | |

## Resource budget

Design targets, to be replaced by measurements (`05-testing.md`): idle CPU under 0.5 % of a core,
memory under 80 MB at level 2, no periodic wake-ups at level 1 other than the indicator's
heartbeat every 2 seconds.
