# Bloom shell · Block D

A dark activity window, contextual pie menu and approval dialog, connected to the existing
Python demo. The UI waits for your actual selection and approval. Context capture and action
execution still use the mocks; no email is written or sent.

## Run the Tauri frontend

Requirements: uv, Node.js 20.19+, npm, Rust/Cargo 1.86+ and the Linux native development
libraries below. On Fedora, install them in your terminal (sudo asks for your password):

```bash
sudo dnf install gtk3-devel webkit2gtk4.1-devel openssl-devel
```

From the repository root:

```bash
make ui-setup
make ui
```

`make ui` launches the Tauri app, with a main window and a transparent pie overlay. `make desktop`
is an equivalent entry point. Vite runs inside the desktop development workflow, so edits to
the frontend reload inside the app. The Python bridge starts alongside it and stops when
the desktop development process exits. Ctrl+C also stops the processes.

When working inside `shell/`, `npm run desktop` runs the same connected desktop workflow.
`npm run dev` starts only Vite; it does not open an app or start the Python bridge.

Click **Start a demo**, choose a suggestion, then approve or reject if asked. The overlay
contains only the circular menu, without a title bar or rectangular panel. On supported
desktops its centre follows the cursor position at invocation, shifting near screen edges
to keep all actions visible. Escape, the centre button, background clicks and focus loss
cancel. Arrow keys navigate, Enter confirms, and 1–3 select directly.

Quick launch registers **Ctrl+Space** globally on Linux/X11 and **Command+Shift+Space** on
macOS. Pressing it again while the pie is visible cancels. Preferences reports registration
failures and unregisters the shortcut when Quick launch is disabled. Approval reveals the
main window; other outcomes hide the overlay without forcing focus onto Bloom.

**GNOME Wayland:** install the optional GNOME 46 desktop adapter once, then use the dedicated
launch target:

```bash
make gnome-setup
# If requested, log out and back in, then enable bloom-desktop@bloom.local.
make ui-gnome
```

The adapter supplies global cursor coordinates and Ctrl+Space, including when Bloom is
minimized. The overlay still uses XWayland for absolute positioning. Preferences reports
whether the GNOME adapter or the limited XWayland fallback is active. See the full
[installation, removal and interface instructions](gnome/README.md). Other platforms do not
need this extension.

The dedicated launch target checks the actual Shell service first. If GNOME's master switch
has disabled user extensions, it stops with activation instructions. Minimize is a native
titlebar control next to Maximize and Close, using GTK's process-local decoration layout on
Linux; no in-page Minimize button or desktop-wide button-layout change is needed.

For a full X11 desktop, or the XWayland fallback without the adapter:

```bash
GDK_BACKEND=x11 make ui
```

This enables the X11 positioning/shortcut implementation; global interception while a native
Wayland app is focused depends on the compositor. GNOME Wayland can also limit the reported
pointer to the last XWayland surface, so this command alone does not guarantee correct
placement over native Wayland apps. Enable the optional GNOME adapter for that case.
A full X11 desktop session provides the supported Linux path. macOS implementation
is present but has not been exercised on a Mac.

Linux cursor capture now queries the X11 desktop root, independent of the main window's
geometry or visibility. Native shortcut launches preserve that captured point rather than
recapturing it in React. A Web Lock keeps each native bridge view eligible for background
work when minimized; it is released on teardown. A native smoke check verified a global
shortcut and completed selection while the main window was minimized. Closing the main
window still exits Bloom; minimizing it leaves the assistant running.

Activity reads the most recent 100 valid entries from `BLOOM_AUDIT_PATH` (default
`.bloom/audit.jsonl`). Expand a row for approval reasons, timings and raw state. Search and
outcome filters operate on these loaded entries. The preview uses actual suggestions from
the latest run; clicking it does not execute anything. Accent, reduced motion and first-slot
orientation are saved locally.

## Native prerequisites and remaining validation

The desktop workflow opens a separate, transparent, undecorated pie overlay when a choice is requested.
Native packaging and a bundled Python runtime are not implemented yet.

Other distributions: follow the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

GTK 3 and WebKitGTK 4.1 must be visible to `pkg-config`. The launcher checks these and gives
an actionable error before starting the desktop app. The browser UI needs neither library.
The native Linux frontend has been compiled and exercised with the installed WebKit driver:
the main window connects, a separate pie window submits a real selection, and the approval
panel can reject without execution. Native screenshots are in `.bloom/previews/`. The driver
cannot synthesize pointer input in this Wayland session, so that smoke check used DOM clicks
in the real webviews. Cursor positioning and global shortcuts are implemented for X11/macOS;
other Wayland compositor integrations, explicit external-app focus restoration, packaged distribution
and macOS validation remain future work.

## Optional browser preview

`make ui-preview` explicitly opens the UI in a browser for layout and integration testing.
It is separate from the Tauri frontend launch. Use the full launch link from the terminal;
its fragment contains a per-launch session token. Opening the bare URL does not authenticate
a new tab.

## Develop the UI

For the Tauri app with hot reload, run `make ui`. For browser-only hot reload, use two terminals:

```bash
# Terminal 1, repository root (run make ui-build once first)
uv run python -m bloom --ui --no-open

# Terminal 2, repository root
npm --prefix shell run dev
```

Open the launch link from terminal 1 with its port changed from `8765` to `1420`, preserving
the `#token=...` fragment. Vite serves the UI; its WebSocket connects to the same Python core.
The default launch port can be changed with `--port` in browser mode. Vite and the native
development wrapper use the default backend port.

## Component boundaries

| File                               | Responsibility                                                      |
| ---------------------------------- | ------------------------------------------------------------------- |
| `src/PieMenu.tsx`                  | Data-driven radial geometry, labels and accessible selection        |
| `src/ApprovalPanel.tsx`            | Exact action preview, approval reasons, expiry and responses        |
| `src/ActivityView.tsx`             | Real run history, filtering and explanations                        |
| `src/bridge.ts`                    | Authentication, connection lifecycle and typed messages             |
| `src/App.tsx`                      | Composition, navigation, preferences and native window coordination |
| `src/native.ts`                    | Native shortcut lifecycle and registration status                   |
| `src-tauri/src/overlay.rs`         | Cursor geometry, monitor bounds and native shortcut registration    |
| `src-tauri/`                       | Main window and transparent native overlay                          |
| `../src/bloom/interface/bridge.py` | Existing Interface implementation and local transport               |

`ActionPlan.candidates` supplies the menu IDs, labels and order. No ranking is reimplemented
in the frontend. Rule weights are not displayed as calibrated confidence percentages.

## IPC

The Python server binds only to `127.0.0.1:8765`, checks Host and Origin and accepts no commands or
history access before authentication. No CORS wildcard is enabled. Launch tokens travel in
the URL fragment and first WebSocket frame, not HTTP query strings or the audit. Tauri's
development webview loads Vite at `127.0.0.1:1420` and connects to this Python server.

1. Connect to `/api/ws`; send `{ "type": "authenticate", "token": "..." }` within 5 seconds.
2. Receive `{ "type": "state", "state": BridgeState }`, initially containing history and phase.
3. Send `{ "type": "start" }` to run the demo. One run is active at a time.
4. In phase `choice`, render `state.plan`, then send a `ChoiceCommand` containing its
   `request_id` and `MenuChoice`. `shown_at` is recorded after the menu has painted.
5. In phase `approval`, render `state.approval`, then send an `ApprovalCommand` with its
   `run_id` and verdict. Editing arguments is deliberately unavailable.
6. Phase `idle` with a new history entry reports the actual finished run.

State snapshots are emitted at the existing Interface interaction points and run boundaries;
they are not a new core event bus. Error messages have `{ "type": "error", "message": "..." }`.
Stale IDs, unoffered actions, duplicate replies and concurrent starts are rejected. Choices
expire after 120 seconds; approvals use the core's `expires_at`. When all clients disconnect,
a pending menu cancels and a pending approval rejects. Reconnection retrieves current state.

## Generated types and checks

```bash
npm --prefix shell run contracts   # regenerate from Pydantic, including D transport models
make check                        # Python checks and real socket integration tests
make ui-check                     # generated types, formatting, TypeScript
npm --prefix shell exec -- playwright install chromium
make ui-test                      # browser → Python → audit, with isolated temporary logs
```

The generated file is committed; never edit `src/contracts.generated.ts` manually. Generation
describes serialized models, whose defaults are always included by `model_dump_json()`.
Browser tests cover selection, approval/rejection, keyboard cancellation, preferences,
separate pie surfaces and narrow layouts. Screenshots are saved in ignored `test-results/`.
Set `BLOOM_CHROMIUM_EXECUTABLE` when using an already installed Chromium binary.
