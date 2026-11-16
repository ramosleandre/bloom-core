# 02 · Sources

A source is the unit of platform-specific work. Adding support for an OS, a desktop or an
application means writing sources, and nothing else.

## The contract

```rust
pub trait Source: Send {
    /// Static facts about this source. Must not touch the OS.
    fn descriptor(&self) -> SourceDescriptor;

    /// Can this source work in this session, right now? Cheap, no side effect, no prompt.
    fn probe(&self, env: &SessionEnv) -> Availability;

    /// Begin watching. Push observations into `sink` until `stop`. Returns quickly;
    /// the source owns whatever thread or run loop its API needs.
    fn start(&mut self, sink: Sink, cfg: &SourceConfig) -> Result<(), SourceError>;

    /// On-demand read against a target. Must return by `deadline`, with whatever it has.
    fn read(&self, req: &ReadRequest, deadline: Instant) -> ReadResult;

    fn stop(&mut self);
}

pub struct SourceDescriptor {
    pub id: SourceId,                 // "<platform>.<name>", e.g. "windows.uia"
    pub channel: Channel,             // used by the precedence table
    pub provides: Vec<Signal>,        // state paths and event kinds it can fill
    pub reads: Vec<ReadKind>,         // deep reads it can answer; may be empty
    pub tier: Tier,                   // A ambient | B focused | C deep (highest it uses)
    pub permission: Option<Permission>,
    pub sensitivity: Sensitivity,     // highest class of data it can emit
}

pub enum Availability {
    Available,
    NeedsPermission { permission: Permission, how_to_grant: String },
    NeedsInstall   { component: String, how_to_install: String },   // e.g. our Shell extension
    Unavailable    { reason: String },                              // wrong session, missing protocol
}
```

Rules every source follows:

- **`probe` never prompts the user.** Permission prompts happen only through the API call
  `permissions.request`, on a user action.
- **`start` is event-driven.** Polling is allowed only where the OS offers no notification (the
  macOS pasteboard), at the slowest rate that works, and stops while the session is idle or locked.
- **`read` honours the deadline.** Accessibility calls into a hung application can block; every
  such call needs a timeout and runs off the engine's threads.
- **Emit facts in OS terms.** Raw application ids, raw roles mapped to the shared `Role` enum (with
  the raw role kept in `raw_role`), no guessing.
- **Mark secure elements and do not read them** (password fields; see `04-privacy-security.md`).
- **Report opt-out signals.** A source that can see that a window is excluded from screen capture,
  or that an entity carries a provider hint, must set `excluded` on it (`07`, `08`).
- **Fail soft.** A source that errors emits `source.state` and stops; it never takes the daemon down.
- **No source talks to another source.** If one needs a fact from another (the accessibility source
  needs the frontmost pid), it reads it from the state handle passed in `SourceConfig`.

## Read kinds

```rust
pub enum ReadKind {
    Current { signals: Vec<Signal> },   // the source's present values for signals it provides
    Selection,            // selected text, exact
    Element,              // focused element: value and attributes
    Surrounding { before: u32, after: u32 },   // text around the caret
    Visible,              // text currently visible in the window
    Document,             // whole document or page, as text (and metadata)
    Structure,            // outline: headings, links, form fields
    Pixels { region: Region },                 // image, PNG bytes
    Ocr    { region: Region },                 // recognized text with boxes
}
```

`Current` is what makes observation level 1 work: a source that is not allowed to watch a signal
continuously must still be able to report it once, when a `capture` asks. Every source implements
`Current` for every signal it lists in `provides`. `SourceConfig` carries the level; a source
subscribes only to the events its level allows, and extension sources receive the level in
`source.welcome`.

A `content.read` from a client names a target and a kind. The engine tries the sources that list
that kind, in channel precedence order, until one answers within the deadline. The result says
which one did. This is the **content ladder**: application extension, then accessibility, then
pixels. The client never chooses the mechanism, only how far down the ladder it allows the engine
to go (`max_channel`).

## Extension sources and the source protocol

Some context is only reachable from inside another program: the browser's tabs and page, the
editor's open file, the shell's working directory, the Wayland compositor's window list. Each of
these is a small extension that **connects out to the daemon** and speaks the source protocol. In
the daemon, a generic `RemoteSource` implements the `Source` trait on top of that connection, so
the engine cannot tell an extension from an in-process source.

Transport: the daemon's local socket (`03-client-api.md`), newline-delimited JSON-RPC 2.0.

```
extension → daemon   source.hello   { id, version, provides, reads, channel, sensitivity }
daemon → extension   source.welcome { accepted, config }           // exclusions, rate limits
extension → daemon   source.observe { patches: [...], events: [...] }   // notification, no reply
daemon → extension   source.read    { request_id, target, kind, deadline_ms }
extension → daemon   source.result  { request_id, ok, data | error }
either direction     ping / pong
```

Why extensions connect out, and never listen:

- An extension that exposes a service (a D-Bus name, a local port) can be called by any process in
  the user's session. An extension that connects to a socket only the user's daemon can create
  exposes nothing.
- One protocol, one authentication check, one place to apply privacy rules.

Bridges for hosts that cannot open a socket:

| Host | Bridge |
|---|---|
| Browser extension | `sense-nmh`, a native-messaging host: stdin/stdout on one side, the daemon socket on the other. Started by the browser. |
| KWin script | Scripts can only call D-Bus. The Linux platform crate owns a small D-Bus object the script calls; it feeds a `RemoteSource`. Inbound only. |
| Shell hooks | A one-line `sense emit` call (the CLI) from `precmd`/`preexec`, or a write to the socket where the shell can do it natively. |

Authentication and trust of sources: `04-privacy-security.md`.

## Registry and lifecycle

At start, and again whenever the session changes (display server, permissions, an extension
connecting):

1. Build `SessionEnv` (OS, version, display server, desktop, available D-Bus names and Wayland
   globals, granted permissions).
2. `probe` every compiled-in source.
3. Start every `Available` source the user's configuration enables.
4. Publish the result as **capabilities** (`03-client-api.md`): for each signal, which source
   provides it or why none does, and what the user could do about it.

When several available sources provide the same signal, all of them start. The precedence table
decides field by field. This is deliberate: on Linux the compositor helper knows the focused
window, AT-SPI knows the focused element, and the browser extension knows the page.

## Adding a source: checklist

1. Add a row to the platform catalog file, with tier, permission and caveats.
2. Implement `Source` in the platform crate (or an extension in `ext/`).
3. Record a fixture with `sense record` and add a replay test (`05-testing.md`).
4. Run the conformance scenarios for the signals it provides.
5. If it emits a new field or event kind: change `sense-schema` first, in its own pull request.
