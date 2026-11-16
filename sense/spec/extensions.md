# Extension sources (the same on every OS)

Extensions run inside another program and connect out to the daemon with the source protocol
(`02-sources.md`). They are the highest-quality sources for the applications they cover, and they
are **portable by construction**: one browser extension serves macOS, Windows and every Linux
desktop. Where the operating system is stingy (Wayland, Electron apps), extensions fill the gap.

Order of value: browser, then shell, then editor, then mail.

General rules:

- An extension translates its host's objects into observations. No logic, no storage, no network.
- It authenticates with the token placed in its configuration at install time.
- It applies the exclusions sent in `source.welcome` at the source, so excluded content never
  leaves the host program.
- It degrades silently when the daemon is absent: reconnect with backoff, never show errors to the
  user of the host program.

## `ext.browser`

One WebExtension (Manifest V3) for Chromium-family browsers and Firefox. Channel `app_extension`.

| Signal | How |
|---|---|
| `browser.windows`, `browser.tabs`, `active_tab`; `tab.activated`, `tab.navigated`, `tab.closed` | `tabs` and `windows` events in the background worker |
| `document.url`, `document.title`, `document.page_meta` | tab data; a content script reads canonical link, Open Graph and JSON-LD metadata |
| `selection.text` (exact), `focus.element` | content script: `selectionchange`, active element descriptor (tag, type, editable, label, `secure` for password and credential fields) |
| reads: Selection, Surrounding, Visible, Document, Structure | content script on request: visible text, readable-article extraction, heading and link outline, form field list (labels only, never values of secure fields) |

- **Bridge.** Browsers only let an extension talk to a local program through native messaging.
  `sense-nmh` is that program: the browser starts it, it relays between stdio and the daemon
  socket. Its manifest is installed per browser and per OS (a JSON file in a browser-specific
  directory on macOS and Linux, a registry key on Windows); the installer owns that table.
- **Correlating with OS windows.** The extension does not know OS window ids. The reducer attaches
  the active tab of the browser's focused window to the OS-level focused window when both report
  the same application as focused and the titles agree.
- **Provider hint.** A page carrying `<meta name="desktop-context" content="none">` is reported as
  excluded (no URL, no content); `content="metadata"` allows URL and title only
  (`08-open-protocol.md`).
- **Private windows.** Not enabled there. If the user enables it, the extension still reports
  `private: true` and no content.
- **Permissions.** Request `tabs`, `nativeMessaging`, `scripting` and host access on demand
  (`activeTab` first; all-sites access as an explicit user choice for ambient selection).
- Page content is untrusted data. The content script never evaluates anything received from the
  daemon; requests are a closed set of read kinds.

## `ext.shell`

Hooks for zsh, bash and fish (`precmd`/`preexec` or equivalents), installed by one line in the
user's shell configuration.

| Signal | How |
|---|---|
| `terminal.cwd`, `terminal.last_command`, `last_exit`, duration; `terminal.command` events | the hook calls `sense emit` (or writes to the socket directly where the shell can) after each command |
| `terminal.vcs` | derived in the core from `cwd` by reading `.git/HEAD` (no command executed) |

The tty is included so the reducer can attach the shell to the terminal window in focus. Commands
are `content`-class data: they can contain secrets typed on the command line. Lines starting with a
space are skipped, following the shell history convention.

## `ext.vscode`

A VS Code extension; it also runs in forks that keep the extension API (Cursor, VSCodium).

| Signal | How |
|---|---|
| `editor.workspace`, `file`, `language`, `selection`, `visible_range` | `window.activeTextEditor`, selection and visible-range change events |
| `editor.diagnostics_count`, `vcs` (branch, dirty) | diagnostics API; the built-in Git extension's API |
| reads: Selection, Surrounding, Visible, Document | the document model, directly |

Node can open Unix sockets and named pipes, so no bridge is needed.

## `ext.mail` (later)

Thunderbird accepts WebExtensions with mail-specific APIs (displayed message: subject, sender,
recipients, body). It reuses the browser extension's transport and bridge. Apple Mail is covered by
`macos.scripting`; Outlook by a later Windows-specific source.

## Compositor-side helpers

`ext/gnome-shell` and `ext/kwin` are extension sources too, but they are Linux-specific and are
specified in `platform-linux.md`.

## Candidates, not planned

JetBrains IDE plugin, LibreOffice (UNO), Obsidian, Slack or Discord (no supported extension API:
they stay on OS-level sources).
