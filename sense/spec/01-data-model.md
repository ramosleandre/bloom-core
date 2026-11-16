# 01 · Data model

Field-complete definitions of every type are in `10-schema-reference.md`; this file explains the
model. The types are defined once, in Rust, in `sense-schema` (`serde` + `schemars`). The crate
emits JSON Schema into `sense/schema/`; Python and TypeScript types are generated from it. CI fails
if the committed schema differs from what the crate emits. Names here are normative; field lists
may grow, and adding an optional field is never a breaking change.

## Observed values

Every leaf the engine serves is wrapped:

```rust
pub struct Observed<T> {
    pub value: T,
    pub via: SourceId,        // e.g. "macos.ax", "linux.gnome", "ext.browser"
    pub at: Timestamp,        // when the source saw it (UTC, ms)
    pub quality: Quality,     // Exact | Ambient | Inferred
    pub sensitivity: Sensitivity, // System | Personal | Content   (set by the core)
    pub truncated: bool,      // cut by a size cap                 (set by the core)
    pub redacted: bool,       // a secret pattern was replaced     (set by the core)
}

pub enum Field<T> {
    Known(Observed<T>),
    Unavailable { reason: Reason },   // serialized as {"unavailable": "<reason>"}
}

pub enum Reason {
    NoSource,           // nothing on this platform or session provides it
    PermissionDenied,   // a source exists but lacks a permission
    NotRequested,       // permission was never asked for
    AppNotAccessible,   // the application does not expose it
    Excluded,           // removed by a privacy rule
    Timeout,            // the source missed the deadline
    NotApplicable,      // e.g. "document" for a calculator
    Level,              // the observation level does not allow it (e.g. timeline at level 1)
    ApprovalDenied,     // the user refused this client for this application
}
```

`Quality` matters to clients: a selection read from the focused element is `Exact`; the Linux
PRIMARY selection is `Ambient` (it is the last selection made anywhere, possibly in another window,
possibly minutes ago); an application id guessed from a process name is `Inferred`.

"Nothing is selected" is `Known(value: "")`. "Could not tell" is `Unavailable`. They must never be
confused.

## State

The state is one tree. Entities have ids that are stable for the daemon's lifetime.

```
State
├── session      os, os_version, desktop, display_server, locale, timezone, input_layout,
│                appearance (light/dark), displays[], power, online
├── activity     idle_ms, locked, in_call (mic or camera in use), do_not_disturb
├── apps[]       AppId → { raw_id, name, pid, exe, canonical, category, launched_at }
├── windows[]    WindowId → { app, title, role, bounds, display, workspace, fullscreen, private }
├── focus        { app, window, element, since }
│     element    { role, label, editable, value_len, caret, multiline, secure }
├── selection    { text, range, owner_window, html? }
├── clipboard    { kinds[], text?, files[], image_meta?, owner_app, concealed }
├── document     { url?, path?, title?, mime?, page_meta? }       for the focused window
├── browser      { windows[], tabs[], active_tab }                when ext.browser is connected
├── editor       { workspace, file, language, selection, visible_range, diagnostics_count, vcs }
├── terminal     { cwd, foreground_command, last_command, last_exit, vcs }
├── media        { player_app, title, artist, album, state }
├── calendar     { current[], next[] }                            events around now
├── notifications recent[]                                        when a source can see them
└── pointer      { x, y, display }
```

Rules:

- **Raw first.** `apps[].raw_id` is exactly what the OS says (bundle id, executable name, desktop
  file id, Wayland `app_id`, X11 `WM_CLASS`). `canonical` and `category` are filled by the core
  from `app-profiles` (below). Sources never fill them.
- **`focus` never points at a client's own window.** A client registers its process ids
  (`client.hello`); the reducer ignores focus changes to those windows and keeps the previous
  target. A client that opens a menu therefore still sees what the user was looking at.
- **`document` is derived.** Several sources may know it (accessibility attribute, browser
  extension, editor extension). The precedence table picks one; the others are dropped.
- **Excluded entities.** An application, window or document can carry `excluded: true` (user
  exclusion, `app-profiles`, a window that refuses screen capture, a private window, a provider
  hint). Its identity is reduced to the application's category; titles, URLs and content are
  `Unavailable(Excluded)`.
- **Secure elements.** If `focus.element.secure` is true, `value`, `selection.text` and any deep
  read of that element are `Unavailable(Excluded)`. Sources must set `secure` and must not read the
  content at all.

## Precedence

When two sources report the same field, the reducer keeps the one that ranks higher in a table that
is data, not code (`sense-core/data/precedence.toml`):

```toml
[focus.app]      order = ["compositor", "window_system", "accessibility"]
[selection.text] order = ["app_extension", "accessibility", "copy", "primary_selection"]
[document.url]   order = ["app_extension", "app_scripting", "accessibility"]
[document.path]  order = ["app_extension", "accessibility", "process_inspection"]
```

Each source declares which **channel** it belongs to (`compositor`, `window_system`,
`accessibility`, `app_extension`, `app_scripting`, `primary_selection`, `copy`, `clipboard`,
`process_inspection`, `pixels`, `system_service`). The table is expressed in channels so that
adding a source needs no change to it. A higher-ranked value older than 2 s loses to a fresher
lower-ranked one for fields that change quickly (`selection`, `focus`).

## Events and the timeline

```rust
pub struct Event {
    pub seq: u64,              // strictly increasing; the cursor clients resume from
    pub at: Timestamp,
    pub via: SourceId,
    pub kind: EventKind,
    pub subject: Option<EntityRef>,
    pub data: serde_json::Value,   // shape fixed per kind, in the schema
}
```

Kinds (initial set):

| Kind | Emitted when |
|---|---|
| `app.launched`, `app.quit` | an application starts or ends |
| `focus.changed` | the focused app, window or element changes (coalesced within 100 ms) |
| `window.opened`, `window.closed`, `window.retitled` | |
| `selection.changed` | debounced 150 ms; carries length, not text |
| `clipboard.changed` | carries kinds and size, and text only if policy allows |
| `document.changed` | the focused window's document changed |
| `tab.activated`, `tab.navigated`, `tab.closed` | from `ext.browser` |
| `file.touched` | a watched folder saw a create, modify or rename |
| `terminal.command` | a command finished: command line, exit code, duration, cwd |
| `media.changed` | |
| `notification.posted` | where the platform lets us see it |
| `activity.idle`, `activity.active`, `activity.locked`, `activity.unlocked` | |
| `source.state` | a source started, stopped, lost a permission |

The timeline is a ring buffer **in memory only**: default 60 minutes or 20,000 events, whichever
is smaller. It is erased on lock (configurable) and on daemon exit. Persistence is not Sense's job:
a client that wants long-term memory subscribes and stores what its own policy allows.

## Snapshot

What `capture` returns: a consistent cut of the state at one `seq`, plus what the client asked for.

```json
{
  "seq": 48213, "at": "2026-11-16T10:12:03.412Z",
  "session":  { "os": "linux", "desktop": "gnome", "display_server": "wayland", "…": "…" },
  "focus": {
    "app":    { "raw_id": "org.mozilla.firefox", "canonical": "firefox", "category": "browser",
                "via": "linux.gnome", "at": "…", "quality": "exact" },
    "window": { "id": "w:71", "title": { "value": "Re: Thursday – Mail", "via": "linux.gnome", "…": "…" } },
    "element": { "role": "text_field", "editable": true, "secure": false, "via": "ext.browser" }
  },
  "selection": { "text": { "value": "Can you confirm Thursday at 2 pm?", "via": "ext.browser",
                           "quality": "exact", "at": "…" } },
  "document":  { "url": { "value": "https://mail.example.org/…", "via": "ext.browser", "…": "…" },
                 "path": { "unavailable": "not_applicable" } },
  "clipboard": { "kinds": ["text"], "owner_app": "a:12", "text": { "unavailable": "excluded" } },
  "recent":    [ { "seq": 48190, "kind": "tab.activated", "…": "…" } ],
  "content":   { },
  "timings_ms": { "linux.gnome": 0, "ext.browser": 6 },
  "target":    "t:w71:e3"
}
```

`target` is the handle for later `content.read` calls (and, outside Sense, for an actuator).

## `app-profiles`

A data file in `sense-core/data/app-profiles.toml`. It is the only place where knowledge about
specific applications lives, so that adding an application is a data change.

```toml
[[app]]
canonical = "firefox"
category  = "browser"
match     = ["org.mozilla.firefox", "firefox.exe", "org.mozilla.Firefox", "firefox", "Navigator.firefox"]
private_window_title = ["Private Browsing"]     # windows matching are marked private
prefers   = ["ext.browser"]                     # skip accessibility for document and selection

[[app]]
canonical = "1password"
category  = "password_manager"
match     = ["com.1password.1password", "1Password.exe", "1password"]
exclude   = "all"                               # see 04-privacy-security.md

[[app]]
canonical = "vscode"
category  = "editor"
match     = ["com.microsoft.VSCode", "Code.exe", "code", "code-url-handler", "Cursor", "VSCodium"]
wake_accessibility = "electron"                 # platform sources know what this means
prefers   = ["ext.vscode"]

[[app]]
canonical = "gnome-terminal"
category  = "terminal"
match     = ["org.gnome.Ptyxis", "org.gnome.Terminal", "gnome-terminal-server"]
copy_shortcut  = "ctrl+shift+c"                 # Ctrl+C would interrupt the running program
paste_shortcut = "ctrl+shift+v"
```

Categories (closed list, extend by pull request): `browser`, `mail`, `chat`, `editor`, `terminal`,
`office`, `notes`, `files`, `media`, `meeting`, `password_manager`, `system`, `other`.
