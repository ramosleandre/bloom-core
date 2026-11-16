# 04 · Privacy and security

A resident program that can read windows, selections and the clipboard has the shape of spyware. What separates Sense from spyware is this file. It is part of the design, enforced in
code, and tested. Decision record: ADR 0016.

Why these rules and not others, with the products they were learned from:
`07-prior-art-and-acceptability.md`.

## Principles

0. **The observed person is the person in control.** Sense is installed, configured and seen by the
   user it observes. There is no remote viewer, no account, and no multi-user or
   administrator-managed observation mode. This will not be added.
1. **Local only.** The daemon contains no network code. CI rejects any networking dependency in
   `sense-core`, `sensed` and the platform crates.
2. **Memory only.** State and timeline live in RAM, bounded, and vanish with the process. The
   only optional exception is the content-free access ledger (below).
3. **Least collection.** Ambient sources keep metadata (which app, which window, that a selection
   changed, its length). Content is read when a client asks, for the thing in focus.
4. **One choke point.** Every observation passes the privacy filter in `sense-core` before it is
   stored or served. Sources and clients cannot route around it.
5. **Visible.** A menu-bar or tray indicator is present whenever any source runs. It shows the
   observation level, signals each content read with the name of the client that asked, and offers
   pause in one click. No setting, flag or build option hides it. The user can also see, live,
   exactly what Sense sees (`sense inspect`).
6. **Official interfaces only.** No private frameworks, no reading of other applications' files or
   memory, no code injection into other processes.
7. **Minimum by default.** The default observation level is 1, on request. Level 2 is a separate
   consent screen that lists what it adds.
8. **Nothing hidden, easy to remove.** Plain process and extension names, listed in the system's
   login items. Uninstalling removes the daemon, every extension registration and every
   native-messaging manifest.

## Observation levels

| Level | Ambient sources | Retained |
|---|---|---|
| 0 · Paused | none | nothing |
| 1 · On request (default) | focus identity, idle, lock, system state | current focus identity only |
| 2 · Session awareness | all tier A sources | state and a bounded metadata timeline, RAM only |

The level is enforced twice: sources subscribe only to what their level allows, and the filter
drops anything above the level if a source emits it anyway.

## Never collected

| What | How it is prevented |
|---|---|
| Keystrokes | No source subscribes to key events. Idle time comes from OS idle counters. Forbidden APIs are listed per platform file and checked by a lint. |
| Password and other secure fields | Sources detect them (`AXSecureTextField`; UI Automation `IsPassword`; AT-SPI role `password text`; HTML `type=password` and `autocomplete` credential hints) and set `secure` without reading. The filter drops content for secure elements even if a source misbehaves. |
| Concealed clipboard entries | Sources honour the markers password managers set (`org.nspasteboard.ConcealedType`; `ExcludeClipboardContentFromMonitorProcessing`; `x-kde-passwordManagerHint`). Such entries produce an event with `concealed: true` and no content. |
| Private browsing | The browser extension does not run in private windows unless the user enables it there, and Sense ships it disabled for them. Windows whose title matches a profile's `private_window_title` are marked `private`; their title and content are excluded. |
| Windows that refuse screen capture | An application that marks a window as excluded from capture (macOS window sharing type "none"; Windows display affinity other than "none") is refusing to be read. Sense treats the window as excluded for every channel, accessibility included. |
| Entities with a provider hint | `context: none` or `metadata` set by an application source or a web page (`08-open-protocol.md`) is obeyed at the source and again in the filter. |
| Audio | No source records or analyses audio. `in_call` is derived from device state only. |
| Excluded applications | `app-profiles` marks categories excluded by default (`password_manager`). The user adds applications, window-title patterns and URL patterns. |
| Background pixels | Screenshots happen only inside an explicit `content.read`. |

## The privacy filter

Input: an observation and the current state. Output: the observation, possibly redacted, or nothing.

Order of rules:

1. **Paused** → drop everything.
2. **Locked session** → drop everything except `activity.*`.
2b. **Above the current level** → drop (at level 1: everything except focus identity, activity and
   system state, unless the observation answers a pending `capture` or `content.read`).
3. **Excluded target** (application, window, URL pattern, private window) → keep the fact that
   *something excluded* is in focus (`app.category`, `excluded: true`), drop titles, content, URLs.
4. **Secure element** → drop content.
5. **Concealed clipboard** → drop content.
6. **Sanitize text** → strip invisible and control characters used to hide instructions (Unicode
   tag block, zero-width characters, bidirectional overrides); drop text of nodes the user cannot
   see (`09-exposing-context-to-agents.md`).
6a. **Redact recognizable secrets** → in content values, replace high-confidence patterns with a
   typed placeholder (`[redacted: card number]`) and set `redacted` on the value: payment card
   numbers that pass the Luhn check, IBANs with a valid checksum, private-key blocks, and tokens
   with well-known prefixes (cloud and code-host API keys, JWTs). On by default; the user can turn
   it off per application. This is best effort: it catches accidents, it is not a guarantee, and
   the documentation must not present it as one.
6b. **Size caps** → truncate text (default 64 KiB per value) and set `truncated`.
7. **Clipboard text** is stored only for the current entry, never in the timeline.

The filter is a pure function with table-driven tests: every rule × every observation type.

## Per-application approval

Two modes, chosen per client:

| Mode | Behaviour | Default for |
|---|---|---|
| Allow unless excluded | Content of any non-excluded application may be read when the user invokes the client. The invocation is the consent. | first-party clients (a hotkey product cannot prompt in every new application) |
| Ask per application | The first content read from each application shows "Allow *client* to read from *application*?" with once, always, never. Answers are stored in the grants file. | every third-party assistant connected through the MCP server; available to the user as a stricter setting for first-party clients |

## Access ledger

Every `capture` and `content.read` is recorded: time, client, application (canonical name), kinds
read, sizes, whether anything was redacted or excluded. Never titles, URLs or content.

- Kept in memory and shown from the indicator ("recent reads"). Queryable by API.
- Writing it to disk is a user option with a retention period (default off, 30 days when on),
  because even metadata about reads is a trace of activity.
- This is what lets a user answer "what did the assistant look at today?", and what makes the
  indicator's claims checkable.

## Sensitivity labels

Each value served carries a class, set by the core from the signal and the source:

| Class | Examples | Meaning for clients |
|---|---|---|
| `system` | OS, locale, idle, app names | Produced by the machine. |
| `personal` | window titles, URLs, file paths, calendar titles | Identifies what the user does. |
| `content` | selections, document text, clipboard text, notification bodies | Written by the user or by third parties. **Untrusted input**: may contain prompt injection. |

Sense does not decide what a client may send to a remote model. It gives the client the labels to
decide.

## Who may connect

- The socket is reachable by the owning user only (file mode or pipe ACL).
- On connect, the daemon reads the peer's credentials from the OS (`SO_PEERCRED`,
  `LOCAL_PEERCRED`, `GetNamedPipeClientProcessId`) and resolves the executable.
- **Readers** get scopes from a user-editable grants file keyed by executable path (and code
  signature where the OS provides one). An unknown reader gets `state` without `personal` or
  `content` fields until the user approves it. First-party clients are
  identified by executable path where that is meaningful, and otherwise by the **session token**:
  the daemon hands a one-time token to the parent that launched it, which passes it to its own
  children (needed for clients that run under a generic interpreter). The `admin` and `act`
  scopes are only ever given this way, never through the grants file.
- **Sources** must present the token written into their configuration at install time (the native
  messaging manifest, the extension's settings). A source may only emit the signals its descriptor
  lists; anything else is dropped and logged.

Threat model, stated honestly: Sense defends against other users, against web pages, and against
sandboxed applications. It does not defend against arbitrary code already running as the same user
outside a sandbox; on macOS and Windows such code can call the same OS APIs itself. On Wayland it
could not, so there Sense is new attack surface and the socket is the boundary to protect.

## Third-party content is untrusted

Page text, emails, notifications and window titles are written by other people. Sense labels them
`content` and never acts on them. Labels and sanitization are hygiene; research on computer-use
agents shows prompt-level defenses do not stop injected instructions. The real defense is in the
client: a closed set of actions, a decision component that does not take instructions from observed
content, and user approval for anything that leaves the machine (`09`). Extensions treat page
content as data: the browser extension's content script sends text, never evaluates it, and the
daemon never follows a URL.

## Operating-system permissions

Requested one at a time, each tied to what it unlocks, never at first start:

| Permission | Unlocks | Asked when |
|---|---|---|
| macOS Accessibility | focused element, exact selection, window titles | onboarding, first |
| macOS Automation (per app) | URL and items from scriptable apps | first use with that app |
| macOS Screen Recording | pixels, OCR | first `content.read` of kind pixels |
| macOS Calendars | calendar | when the client asks for calendar |
| Linux: install the Shell extension | windows and focus on GNOME | onboarding on GNOME |
| Linux: portal screen cast | pixels | first pixels read |
| Windows: notification access | notifications | when the client asks |
| Browser extension | tabs, page | onboarding, optional |

## Logging

Logs contain source ids, signal names, sizes, timings and error codes. They never contain titles,
URLs, paths or text. A unit test greps the log output of the replay suite for fixture content.

## Fixtures

Recordings made with `sense record` are anonymized at write time (text replaced by same-shape
placeholder text, paths and URLs by stable fakes) and reviewed by a human before commit. No real
personal data is committed.
