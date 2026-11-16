# 10 · Schema reference

Field-complete definitions for package 0.1 (`sense-schema`). `01-data-model.md` explains the model;
this file is what to type. Where the two disagree, this file wins and the other is a bug.

Conventions:

- JSON field names are `snake_case`. Enum values serialize as `snake_case` strings.
- `F<T>` is shorthand for `Field<T>`: either an observed value with its envelope, or
  `{"unavailable": "<reason>"}`. Every leaf that comes from outside the engine is an `F<T>`.
- Plain types (no `F`) are produced by the core itself and are always present.
- `Option<T>` means the key may be absent; it is only used for things that do not apply.
- Timestamps are RFC 3339 strings in UTC with milliseconds. Durations are integer milliseconds.
- Ids are short opaque strings, stable for the life of the entity inside one daemon run. Prefixes
  are for humans only: `a:` application, `w:` window, `e:` element, `d:` display, `t:` target.
- All text is UTF-8, sanitized and capped by the core (`04`, `09`).

## Envelope

```rust
struct Observed<T> {
    value: T,
    via: SourceId,            // "linux.gnome"
    at: Timestamp,
    quality: Quality,
    sensitivity: Sensitivity,
    truncated: bool,          // default false; omitted from JSON when false
    redacted: bool,           // default false; omitted from JSON when false
}
enum Field<T> { Known(Observed<T>), Unavailable { unavailable: Reason } }

enum Quality     { Exact, Ambient, Inferred }
enum Sensitivity { System, Personal, Content }
enum Reason      { NoSource, PermissionDenied, NotRequested, AppNotAccessible, Excluded,
                   Timeout, NotApplicable, Level, ApprovalDenied }
```

## Identifiers and enums

```rust
type SourceId  = String;      // "<platform>.<name>": platform ∈ macos | windows | linux | ext
type Timestamp = String;
type Signal    = String;      // a state path ("selection.text") or an event kind ("tab.activated")

enum Channel   { Compositor, WindowSystem, Accessibility, AppExtension, AppScripting,
                 PrimarySelection, Copy, Clipboard, ProcessInspection, Pixels, SystemService }
enum Tier      { Ambient, Focused, Deep }
enum Level     { Paused, OnRequest, SessionAwareness }             // 0, 1, 2

enum Os            { Macos, Windows, Linux }
enum DisplayServer { Quartz, Win32, X11, Wayland }
enum SessionFamily { X11, Gnome, Kde, Wlroots, Unknown }           // Linux only

enum Category  { Browser, Mail, Chat, Editor, Terminal, Office, Notes, Files, Media, Meeting,
                 PasswordManager, System, Other }

enum Role      { Window, Dialog, Document, WebArea, TextField, TextArea, SecureTextField,
                 Button, Link, Checkbox, Radio, ComboBox, Menu, MenuItem, Tab, List, ListItem,
                 Table, Row, Cell, Heading, Paragraph, Image, Toolbar, Terminal, Canvas,
                 Group, Unknown }
enum ExcludedBy { User, AppProfile, CaptureProtected, PrivateWindow, ProviderHint }
```

`Role` is deliberately small. Sources map native roles onto it and keep the native name in
`raw_role`. Anything without a mapping is `Unknown`, never a guess.

## State

```rust
struct State {
    seq: u64,
    level: Level,
    session: Session,
    activity: Activity,
    apps: Map<AppId, App>,
    windows: Map<WindowId, Window>,
    focus: Focus,
    selection: Selection,
    clipboard: Clipboard,
    document: Document,
    pointer: F<Pointer>,
    browser: Option<Browser>,       // present when a browser source is connected
    editor: Option<Editor>,
    terminal: Option<Terminal>,
    media: Option<Media>,
    calendar: Option<Calendar>,
    notifications: Option<Vec<Notification>>,
}

struct Session {
    os: Os, os_version: String,
    display_server: DisplayServer,
    desktop: Option<String>,        // "gnome", "kde", "sway", …  (Linux)
    desktop_version: Option<String>,
    family: Option<SessionFamily>,  // Linux
    locale: F<String>,              // BCP 47
    timezone: F<String>,            // IANA
    input_layout: F<String>,
    appearance: F<Appearance>,      // Light | Dark
    displays: Vec<Display>,
    power: F<Power>,
    online: F<bool>,
}
struct Display { id: DisplayId, bounds: Rect, scale: f32, primary: bool }
struct Power   { on_battery: bool, percent: Option<u8> }
struct Rect    { x: i32, y: i32, w: u32, h: u32 }         // logical pixels, global space

struct Activity {
    idle_ms: F<u64>,
    locked: F<bool>,
    in_call: F<bool>,               // microphone or camera in use
    do_not_disturb: F<bool>,
}

struct App {
    id: AppId,
    raw_id: String,                 // exactly what the OS reports
    name: String,
    pid: u32,
    exe: Option<String>,            // sensitivity Personal; may be absent
    canonical: Option<String>,      // from app-profiles
    category: Category,             // Other when unknown
    launched_at: Option<Timestamp>,
    excluded: Option<ExcludedBy>,
    via: SourceId,
}

struct Window {
    id: WindowId,
    app: AppId,
    title: F<String>,               // Personal
    role: Role,                     // Window | Dialog
    bounds: F<Rect>,                // unavailable on most Wayland sessions
    display: Option<DisplayId>,
    workspace: Option<String>,
    fullscreen: bool,
    private: bool,                  // private-browsing window
    excluded: Option<ExcludedBy>,
    via: SourceId,
}

struct Focus {
    app: Option<AppId>,             // None when nothing is focused or the session is locked
    window: Option<WindowId>,
    element: F<Element>,
    since: Timestamp,
    target: Option<TargetId>,
}
struct Element {
    id: ElementId,
    role: Role, raw_role: String,
    label: Option<String>,          // Personal
    editable: bool,
    multiline: bool,
    secure: bool,
    value_len: Option<u32>,         // characters; never the value itself
    caret: Option<u32>,             // character offset
}

struct Selection {
    text: F<String>,                // Content; Known("") means "nothing selected"
    len: F<u32>,                    // available at level 2 without reading the text
    range: Option<TextRange>,       // offsets inside the focused element, when known
    owner_window: Option<WindowId>, // None when the owner cannot be established (Wayland PRIMARY)
    html: Option<F<String>>,        // only from sources that have it, on request
}
struct TextRange { start: u32, end: u32 }

struct Clipboard {
    kinds: F<Vec<ClipKind>>,        // Text | Html | Image | Files | Other
    size: F<u64>,                   // bytes of the preferred representation
    text: F<String>,                // Content; only with scope clipboard.text, only current entry
    files: F<Vec<String>>,          // Personal
    owner_app: Option<AppId>,
    concealed: bool,
    changed_at: Option<Timestamp>,
}

struct Document {                   // for the focused window; derived by precedence
    url: F<String>,                 // Personal
    path: F<String>,                // Personal
    title: F<String>,               // Personal
    mime: F<String>,
    page_meta: Option<F<PageMeta>>,
    excluded: Option<ExcludedBy>,
}
struct PageMeta { canonical_url: Option<String>, description: Option<String>,
                  site_name: Option<String>, schema_types: Vec<String> }

struct Pointer { x: i32, y: i32, display: Option<DisplayId> }
```

Optional subtrees:

```rust
struct Browser  { windows: Vec<BrowserWindow>, active_tab: Option<TabId> }
struct BrowserWindow { id: String, os_window: Option<WindowId>, focused: bool, private: bool,
                       tabs: Vec<Tab> }
struct Tab      { id: TabId, url: F<String>, title: F<String>, active: bool, audible: bool,
                  excluded: Option<ExcludedBy> }

struct Editor   { workspace: F<String>, file: F<String>, language: F<String>,
                  selection: Option<LineRange>, visible_range: Option<LineRange>,
                  diagnostics_count: F<u32>, vcs: Option<Vcs> }
struct LineRange { start_line: u32, start_col: u32, end_line: u32, end_col: u32 }
struct Vcs      { repo_root: String, branch: Option<String>, dirty: Option<bool> }

struct Terminal { os_window: Option<WindowId>, tty: Option<String>, cwd: F<String>,
                  foreground_command: F<String>,   // Content
                  last_command: F<String>,         // Content
                  last_exit: F<i32>, last_duration_ms: F<u64>, vcs: Option<Vcs> }

struct Media    { player_app: Option<AppId>, title: F<String>, artist: F<String>,
                  album: F<String>, state: F<PlayState> }        // Playing | Paused | Stopped

struct Calendar { current: Vec<CalEvent>, next: Vec<CalEvent> }  // next: at most 5, within 24 h
struct CalEvent { title: F<String>, start: Timestamp, end: Timestamp, all_day: bool,
                  location: F<String>, calendar: Option<String>, attendees_count: Option<u32> }

struct Notification { app: Option<AppId>, title: F<String>, body: F<String>,   // body: Content
                      at: Timestamp }
```

## Sensitivity of each field

Set by the core, not by sources. `System` unless listed:

| Sensitivity | Fields |
|---|---|
| Personal | `window.title`, `element.label`, `document.*`, `tab.url`, `tab.title`, `clipboard.files`, `app.exe`, `editor.workspace`, `editor.file`, `terminal.cwd`, `media.*`, `calendar.*`, `notification.title` |
| Content | `selection.text`, `selection.html`, `clipboard.text`, `terminal.foreground_command`, `terminal.last_command`, `notification.body`, everything returned by `content.read` |

## What exists at each level

| Subtree | Level 1 (in state between captures) | Level 1 (inside a capture response) | Level 2 |
|---|---|---|---|
| `session`, `activity` | yes | yes | yes |
| `focus.app`, `focus.window` ids; `apps` entry of the focused app without `exe` | yes | yes | yes |
| `windows[*].title`, `focus.element`, `selection`, `document`, `pointer` | no | read on demand | yes (selection text still on demand; `len` kept) |
| `clipboard` | no | kinds and size on demand; text with scope | kinds and size kept; text with scope |
| `browser`, `editor`, `terminal`, `media`, `calendar`, `notifications` | no | on demand, focused application only | yes |
| timeline | no | `recent` is `Level` | yes |

## Events

```rust
struct Event { seq: u64, at: Timestamp, via: SourceId, kind: String,
               subject: Option<EntityRef>, data: EventData }
enum EntityRef { App(AppId), Window(WindowId), Tab(TabId), Element(ElementId) }
```

`data` by kind (all fields are plain metadata; no event carries content):

| Kind | Data |
|---|---|
| `app.launched`, `app.quit` | `{ raw_id, canonical?, category }` |
| `focus.changed` | `{ from_app?, to_app?, to_window?, to_role? }` |
| `window.opened`, `window.closed` | `{ app }` |
| `window.retitled` | `{ }` (the title is in the state, not in the event) |
| `selection.changed` | `{ len, quality }` |
| `clipboard.changed` | `{ kinds, size, owner_app?, concealed }` |
| `document.changed` | `{ has_url, has_path }` |
| `tab.activated`, `tab.navigated`, `tab.closed` | `{ tab, private }` |
| `file.touched` | `{ op: created \| modified \| renamed \| removed, dir_id }` (the path is Personal: resolved through `state`, not stored in the event) |
| `terminal.command` | `{ exit, duration_ms }` |
| `media.changed` | `{ state }` |
| `notification.posted` | `{ app? }` |
| `activity.idle`, `activity.active`, `activity.locked`, `activity.unlocked` | `{ }` |
| `source.state` | `{ source, state: running \| stopped \| needs_permission \| needs_install \| error, code? }` |
| `level.changed` | `{ from, to }` |

Keeping titles, paths and commands out of events means the timeline is safe to retain and to show.
A client that needs the title of a window named in an event reads it from the state while the
window exists.

## Snapshot and reads

```rust
struct Snapshot {
    seq: u64, at: Timestamp, level: Level,
    session: Option<Session>, activity: Option<Activity>,
    focus: Option<FocusView>,           // focus with the app and window records inlined
    selection: Option<Selection>, clipboard: Option<Clipboard>, document: Option<Document>,
    pointer: Option<F<Pointer>>,
    browser: Option<Browser>, editor: Option<Editor>, terminal: Option<Terminal>,
    media: Option<Media>, calendar: Option<Calendar>,
    recent: F<Vec<Event>>,
    content: Vec<ReadResult>,
    timings_ms: Map<SourceId, u32>,
    target: Option<TargetId>,
}
struct FocusView { app: Option<App>, window: Option<Window>, element: F<Element>, since: Timestamp }

struct ReadRequest { target: TargetId, kind: ReadKind, max_bytes: u32, max_tokens: Option<u32> }
enum   ReadKind    { Current { signals: Vec<Signal> }, Selection, Element,
                     Surrounding { before: u32, after: u32 }, Visible, Document, Structure,
                     Pixels { region: Region }, Ocr { region: Region } }
enum   Region      { Element, Window, Rect(Rect) }

struct ReadResult {
    kind: String,
    result: F<ReadData>,
    elapsed_ms: u32,
}
enum ReadData {
    Text  { text: String },
    Tree  { nodes: Vec<Node> },                    // already reduced (09, rule 3)
    Image { png_base64: String, width: u32, height: u32 },
    Ocr   { lines: Vec<OcrLine> },
    Signals { patch: serde_json::Value },          // for ReadKind::Current
}
struct Node    { r#ref: String, role: Role, label: Option<String>, text: Option<String>,
                 value: Option<String>, level: Option<u8>, href: Option<String>,
                 children: Vec<Node> }
struct OcrLine { text: String, bounds: Rect, confidence: f32 }
```

## Capabilities

```rust
struct Capabilities { level: Level, signals: Map<Signal, SignalStatus>, sources: Vec<SourceStatus> }
enum SignalStatus {
    Available       { via: Vec<SourceId>, note: Option<String> },
    NeedsPermission { source: SourceId, permission: String, how: String },
    NeedsInstall    { component: String, how: String },
    Unavailable     { reason: String },
}
struct SourceStatus { id: SourceId, state: String, version: Option<String>, channel: Channel }
```

## Access ledger entry

```rust
struct AccessEntry { at: Timestamp, client: String, method: String,       // capture | content.read
                     app: Option<String>,                                 // canonical or category
                     kinds: Vec<String>, bytes: u64,
                     redacted: bool, excluded: bool, approved_by: ApprovalMode }
enum ApprovalMode { Invocation, Asked, Remembered }
```

## Decisions made here that the other files did not spell out

- Events carry no titles, paths, URLs or command lines.
- `selection.len` exists separately from `selection.text`, so level 2 can know that something is
  selected without reading it.
- `window.bounds` and `pointer` are `F<…>` because most Wayland sessions cannot provide them.
- `app.category` is always present (`Other`), so an excluded application can still be described.
- `Snapshot.focus` inlines the application and window records, so a client needs no second lookup.
