# 00 · Overview

## Goal

Give a client the fullest possible picture of what the user is doing, from everything the
operating system and its applications expose, with the same data model on macOS, Windows and Linux.

"Fullest possible" has three dimensions, and the design must serve all three:

- **Breadth**: many kinds of signal (windows, focus, selection, clipboard, documents, browser tabs,
  editor state, terminal state, media, calendar, notifications, idle state, system state).
- **Depth**: for the thing in focus, as much content as the client asks for (the selection, the
  field, the visible text, the whole document).
- **Time**: not only "now" but "what led here": the last applications, copies, files and pages.

## Non-goals

- Acting on the system (inserting text, placing windows, global hotkeys). That is a sibling
  service, specified in `12-acting.md`. It runs in the same daemon because it needs the same OS
  permissions and the same compositor-side helpers, but it has its own API namespace, its own
  scope, and is never exposed to third-party assistants. Everything called "Sense" is read-only.
- Understanding. Sense does not summarize, classify intent or rank anything.
- Keystroke capture, network traffic inspection, reading other applications' private databases
  or memory, and continuous screen recording. See `04-privacy-security.md`.
- Private or undocumented OS interfaces. If a signal is only reachable that way, the catalog marks
  it "not available" for that platform.

## Why a resident engine

The obvious design is a library that reads the screen, from a cold start, when the client asks.
It has three problems:

- A client that opens its own window **changes the context** it wants to read: focus moves, and on
  some systems the selection becomes unreadable. A resident engine already knows which window was
  in front, at every observation level.
- A cold read pays every source's start-up cost (connections, permission checks, extension
  handshakes) at the worst moment. A resident engine keeps its sources warm.
- Operating systems expose much of their context only as **events** (focus changed, clipboard
  changed, file opened, tab activated). What was not listened to cannot be queried afterwards. A
  resident engine can keep that recent past, for users who opt in.

So Sense runs for the whole user session and does two things:

1. **Watch**: cheap, event-driven sources keep a live state and, if the user opted in, a short
   timeline in memory.
2. **Read on demand**: expensive or intrusive sources run only when a client asks, against a target
   the watch tier already identified.

How much is watched is the user's choice, and the default is the minimum (see "Observation levels"
below). Decision record: ADR 0013.

## Architecture

```
                 ┌──────────────────────── sensed (one per user session) ────────────────────────┐
 in-process      │                                                                                │
 sources         │   Source ──┐                                                                   │
 (native APIs)   │   Source ──┤  observations   ┌──────────┐  patches  ┌─────────┐                │
                 │   Source ──┼────────────────▶│ privacy  │──────────▶│ reducer │──▶ state       │   clients
 extension       │            │                 │  filter  │           └─────────┘──▶ timeline    │  (Bloom, CLI,
 sources         │   Remote ──┘                 └──────────┘                 ▲                    │   inspector)
 (browser,       │   Source                                                  │                    │
 compositor,     │     ▲                              read requests ─────────┘                    │
 editor, shell)  │     │ source protocol                                   ┌────────────┐         │
 ────────────────┼─────┘                                                   │ API server │◀────────┼──── local socket
                 │                                                         └────────────┘         │
                 └────────────────────────────────────────────────────────────────────────────────┘
```

- A **source** is one way of observing: one OS API family, or one extension. It declares what it
  provides, pushes observations, and answers read requests. (`02-sources.md`)
- The **privacy filter** applies exclusions and redaction to every observation before it reaches
  anything else. (`04-privacy-security.md`)
- The **reducer** merges observations into the **state** (what is true now) and appends to the
  **timeline** (what happened recently). When two sources report the same field, a precedence table
  decides. (`01-data-model.md`)
- The **API server** serves clients over a local socket: read state, capture a snapshot, read
  content in depth, subscribe to events, inspect capabilities and permissions. (`03-client-api.md`)

Decision records: ADR 0012 (standalone daemon, Rust), ADR 0014 (one source contract).

## How per-OS differences are contained

| Difference | Where it is absorbed |
|---|---|
| Different native APIs for the same signal | One source per API family, in the platform crate. Same observation types out. |
| A signal exists on one OS only | The source simply does not exist elsewhere. Clients see it in `capabilities`. |
| Linux desktops expose different doors | Several sources for the same signal; run-time detection starts the ones that work (ADR 0015). |
| A signal needs code inside another program | An extension source in `ext/`, speaking the same source protocol as everything else. |
| Permissions differ | Each source reports `granted`, `denied`, `not_requested` or `unavailable`, with how to fix it. |
| Application ids differ (`com.apple.mail`, `thunderbird.exe`, `org.mozilla.Thunderbird`) | Sources report raw ids. A data file (`app-profiles`) maps them to a canonical name and category in the core. |

Nothing above the source layer branches on the operating system.

## The three tiers of observation

Every signal in the catalogs is assigned one tier. The tier fixes when it runs.

| Tier | Runs | Budget | Examples |
|---|---|---|---|
| **A · ambient** | event-driven, as far as the observation level allows | negligible CPU, no user-visible effect | frontmost app and window, selection change, clipboard change, idle, media, tab activated |
| **B · focused** | on a client's `capture` | under 50 ms | focused element attributes, selected text through accessibility, document path or URL |
| **C · deep** | only on an explicit `content.read` | up to seconds, may need extra permission | accessibility subtree text, full page text, screenshot and OCR |

## Observation levels

Tiers say what a source costs. Levels say what the user has allowed. The reasoning, from how
comparable products were received, is in `07-prior-art-and-acceptability.md`.

| Level | Name | Ambient sources running | Kept in memory |
|---|---|---|---|
| 0 | Paused | none | nothing |
| 1 | **On request** (default) | only focus identity (which app and window are in front), idle and lock, system state | the current focus identity; no titles, no timeline |
| 2 | Session awareness (opt-in) | all tier A sources | state plus a bounded metadata timeline, in RAM |

At level 1, titles, selection, clipboard, document and application-extension data are read at the
moment of a `capture`, through the sources' `read` path, and are not retained afterwards. At every
level, tier C runs only on an explicit request, no observed context is written to disk, and a
visible indicator shows that Sense is running.

## Glossary

- **Source**: an adapter that observes one thing through one mechanism.
- **Observation**: what a source emits: a typed patch to the state, or an event.
- **State**: the engine's current picture. A tree of entities.
- **Timeline**: a bounded, in-memory log of recent events.
- **Snapshot**: a consistent copy of part of the state and timeline, returned by `capture`.
- **Target**: an opaque id for an application, window or element, valid while that thing exists.
- **Session family** (Linux): X11, GNOME, KDE or wlroots; decides which sources can work.
- **Compositor-side helper**: our code running inside the desktop shell (GNOME Shell extension,
  KWin script), because on Wayland only the compositor knows about other applications' windows.
