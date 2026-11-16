# 06 · Work packages

Each package fits one focused agent session and ends with something runnable. Every brief given to
an agent includes: the invariants in `sense/README.md`, the spec files listed under **Read**, and
the **Done when** line as the acceptance test. Packages in the same stage are independent.

## The minimum useful slice

The full plan below is large. What a first client (Bloom's menu) needs is small: at the user's
request, the application, the window, the selection, and the URL or file. That is:

- the spike (stage S), in parallel with
- stage 0 (the engine on fake sources),
- two platform packages from stage 1: `linux.gnome` (1.2) and macOS `workspace` + `ax` (1.4), with
  `linux.x11` (1.1) for automated tests,
- the browser extension (2.1),
- the acting service on those same platforms (A.1 to A.3),
- the indicator (5.1),
- Bloom's side (B.1, B.2),

all at observation level 1. Everything else (session awareness, other Linux families, Windows,
calendar, files, media, pixels, the MCP server, the public profile) adds breadth and can wait
without redesign. Build the slice first; each later package is then an independent addition.

## Stage S · Verification spike (throwaway code)

| # | Package | Read | Done when |
|---|---|---|---|
| S | Answer the remaining questions (S2b, S3 to S7). S1 and S2 were answered on 16 November 2026 (`sense/spike/gnome-shell/`) | 14 | one page of results in `lab/results/`; each open question in `14-open-questions.md` moved to "closed" with its consequence applied to the specs |

Nothing in stage 0 waits for the spike. Stage 1 and the acting packages do.

## Stage 0 · Foundations (no OS code)

| # | Package | Read | Deliver | Done when |
|---|---|---|---|---|
| 0.1 | Schema | 01, 10 | `sense-schema`: all types, serde, JSON Schema emission; `schema/` committed; drift check | round-trip tests pass for every type; the example snapshot in 01 validates |
| 0.2 | Core engine | 00, 01, 02 | `sense-core`: `Source` trait, registry, reducer, precedence table, state, timeline; `sense-fake` | scripted-source tests cover precedence, staleness, coalescing, own-window exclusion, timeline bounds |
| 0.3 | Privacy filter and levels | 04, 07 | the filter as a pure function, observation levels, `app-profiles` loader, exclusions, provider hints, pause, lock | table-driven tests: every rule × every observation type × every level |
| 0.3b | Redaction, approvals, ledger | 04 | secret-pattern redaction, per-application approval modes and grants file, the access ledger and `access.log` | scenarios 15, 16, 17 against fake sources |
| 0.4 | API server and CLI | 03, 11 | socket server, handshake, scopes, session token, all methods; `sensed` with parent watching and config files; `sense` CLI | `sense capture` against `sense-fake` prints a valid snapshot on Linux, macOS and Windows |
| 0.5 | Remote sources | 02 | `RemoteSource`, source protocol, token check | a 30-line test script connecting as a source can feed state and answer a read |
| 0.6 | Replay and record | 05 | `sense record`, anonymizer, replay harness | one synthetic fixture replays to an expected state file in CI |
| 0.7 | Rendering for models | 09 | text sanitization, the `card` rendering, the structure reduction with a token budget | fixture snapshots render to expected cards; hidden-character and off-screen fixtures are stripped; a large tree fixture fits its budget |

After stage 0, the whole engine works end to end with fake sources, and clients can be written.

## Stage 1 · "Now" on each platform (tier A and B)

| # | Package | Read | Deliver | Done when |
|---|---|---|---|---|
| 1.1 | Linux session detection and `linux.x11` | platform-linux | `SessionEnv`, family detection, the X11 source; fixture app; conformance in CI | scenarios 1, 2, 4, 6, 9 green in CI |
| 1.2 | `linux.gnome` | platform-linux | the Shell extension and its daemon side | scenarios 1, 2, 4, 6 on GNOME 46 and current; clean `NeedsInstall` without it |
| 1.3 | `linux.atspi` | platform-linux | focus element, exact selection, reads | scenarios 2, 3, 5 with the GTK fixture |
| 1.4 | macOS: `workspace`, `windows`, `ax` | platform-macos | the three sources, signing setup for development | scenarios 1, 2, 3, 5, 6, 7 on a Mac |
| 1.5 | macOS: `pasteboard`, `activity`, `system` | platform-macos | | scenarios 4, 9 |
| 1.6 | Windows: `shell`, `uia`, `clipboard`, `activity`, `system` | platform-windows | | scenarios 1 to 7, 9 |
| 1.7 | Linux family-independent: `login1`, `idle`, `system`, `mpris`, `pipewire` | platform-linux | | replay fixtures; lock clears state |

## Stage A · Acting service

| # | Package | Read | Deliver | Done when |
|---|---|---|---|---|
| A.1 | Acting core | 12 | `hands-core`: the effect types, target verification, clipboard save and restore logic, ledger entries, the `act` scope; a fake platform | scenarios A4, A5, A7 against the fake |
| A.2 | Acting on GNOME and X11 | 12, 13, platform-linux | `focus.restore`, `insert_text`, `selection.copy`. Hotkey, pointer and menu placement already exist in block D's prototype (`feature/tauri-ui`): reuse them, add only the portal path for GNOME 48+ and `sense trigger` | scenarios A1, A2, A3, A6 |
| A.3 | Acting on macOS | 12, platform-macos | hotkey, `insert_text` through accessibility and paste; a note for block D on the non-activating panel | scenarios A1, A2, A3 |
| A.4 | Acting on Windows, KDE, wlroots | 12 | later, with their reading sources | same scenarios |

## Stage B · Bloom's side

| # | Package | Read | Deliver | Done when |
|---|---|---|---|---|
| B.1 | Contract change | 13 | the `contract` pull request: `Seen`, `AppRef`, new `Observation`, `requires` / `wants`; mocks, rules backend and sample dataset updated | `make check` green; reviewed by the four owners |
| B.2 | `SenseSurface` and `ReplaySurface` | 13 | the mapping in one file; `app.py` picks the real surface when a session token is present | `make run` replays a recorded snapshot from each tier-1 platform; works with no daemon present |
| B.3 | Shell wiring | 11, 12, 13 | the shell starts `sensed`, passes the token, binds the hotkey, places the menu | the slice runs end to end on GNOME and macOS |

## Stage 2 · Application extensions

| # | Package | Read | Deliver | Done when |
|---|---|---|---|---|
| 2.1 | `ext.browser` and `sense-nmh` | extensions | the extension, the bridge, manifest installation for Chromium and Firefox on each OS | browser scenarios pass with the static fixture page; tab correlates to the OS window |
| 2.2 | `ext.shell` | extensions | hooks for zsh, bash, fish; `sense emit` | `terminal.*` filled after a command, attached to the focused terminal window |
| 2.3 | `ext.vscode` | extensions | the extension | `editor.*` filled; Document read returns the buffer |

## Stage 3 · Remaining Linux families and breadth

| # | Package | Read | Done when |
|---|---|---|---|
| 3.1 | `linux.kwin` and data-control | platform-linux | scenarios 1, 2, 4, 6 on Plasma 6 |
| 3.2 | `linux.wlr` | platform-linux | same on Sway; partial capability on a compositor lacking one protocol |
| 3.3 | Calendar: `macos.eventkit`, `linux.calendar`, `windows.calendar` | platform files | `calendar.current/next` filled from a test calendar |
| 3.4 | Files: `*.fs`, `linux.recent`, `macos.spotlight`, `windows.explorer` | platform files | `file.touched` and recent documents in the timeline |
| 3.5 | Media, notifications, `in_call` on macOS and Windows | platform files | signals filled or correctly `Unavailable` |
| 3.6 | `macos.scripting` | platform-macos | Safari and Mail without the browser extension |

## Stage 4 · Pixels

| # | Package | Done when |
|---|---|---|
| 4.1 | `macos.screen`, `windows.screen`, `linux.portal.screen` with OCR | `content.read` kind Ocr returns the fixture text for an application with no accessibility; never runs without a request |

## Stage 5 · Product surface

| # | Package | Done when |
|---|---|---|
| 5.1 | Indicator (`sense-tray`, and the top-bar indicator in the GNOME extension), transparency view and pause (`11`) | scenario 14 passes; killing the indicator stops every source within 5 seconds on each platform; the indicator cannot be disabled. **Required before any build is given to a user outside the team.** |
| 5.2 | Onboarding and consent, driven by `capabilities` | nothing is observed before consent; each permission or component shows what it unlocks; level 2 has its own screen |
| 5.3 | Installers and uninstallers | signed helper on macOS, package identity on Windows, native packages and extension distribution on Linux; uninstall leaves no registered component |

## Stage 6 · Interoperability and publication

| # | Package | Read | Done when |
|---|---|---|---|
| 6.1 | `sense-mcp` | 08 | an MCP client lists the resources and calls `capture_context`; off by default; per-assistant scopes |
| 6.2 | Provider hints | 08 | scenario 13 passes; documented for application and web authors |
| 6.3 | Desktop-context profile on MCP | 08 | schema, MCP conventions and rules of conduct extracted into one short document with the conformance suite. Budget: days. Publication in a public repository only after the team's decision |

## What needs a human decision before work starts

1. Accept or amend ADRs 0012 to 0018 (the list of decisions, with recommendations, is in
   `14-open-questions.md`).
2. Who owns which platform's conformance runs (macOS needs the team's Mac).
3. The component's name.
4. The `Observation` contract change proposed in `13-bloom-integration.md`.
