# D · Connected interface validation

Date: 2026-10-23. Scope: the Linux-first Tauri frontend, browser preview and Python bridge.
Decision: [ADR 0007](../adr/0007-interactive-shell.md).

## What was exercised

- Suggestions are produced by the existing RulesDecision and DecisionRanker, passed through
  the existing pipeline, and rendered from ActionPlan. No UI-side action ranking exists.
- Selection resumes that actual pipeline. Choosing a read-only summary finishes without an
  approval. Choosing a reply draft requests approval because the context is untrusted.
- Approval completes the simulated action; rejection records cancellation with no result.
- Closing the menu, stale replies, unoffered actions, expired approvals, wrong session tokens,
  unrelated origins/hosts and losing the UI connection are covered by Python bridge tests.
- Browser tests verify the final audit record, arrow/Enter navigation, quick launch, Escape,
  persisted preferences, non-executing preview interactions, a separate pie surface and a
  390-pixel-wide viewport. The first browser test also checks for JavaScript runtime errors.

## Checks run

| Check | Result |
|---|---|
| `make check` | Passed: format, lint, pyright, all four import contracts, 51 Python tests |
| `make ui-check` | Passed: generated contracts, Prettier, TypeScript |
| `make ui-build` | Passed: generated contracts and production frontend |
| `npm --prefix shell test` | 7 browser tests passed against a real local Python process |
| Terminal `make run` | Passed using an isolated temporary audit path |
| Rust dependency resolution | Cargo.lock generated for Rust 1.86-compatible dependencies |
| `cargo check --locked` | Passed against the actual GTK/WebKit development libraries |
| `cargo build --locked` | Passed: native Tauri executable built |
| Native WebKit smoke test | Main window connected, separate pie selection completed, approval rejected with no execution |

The environment is Fedora 40, Python 3.13, Node 20.19 and Rust 1.86. Browser checks used an
already installed Chromium through `BLOOM_CHROMIUM_EXECUTABLE`. Screenshots of the overview,
pie menu, approval panel and narrow layout were inspected and remain in ignored
`shell/test-results/`. These checks validate UI behavior; they are not the official macOS
latency or decision-quality measurements.

## Native validation and remaining work

GTK 3 and WebKitGTK 4.1 development packages became available during implementation. The
actual native Tauri app was built with Rust 1.86 and controlled through the installed
WebKitWebDriver, with `TAURI_WEBVIEW_AUTOMATION=true` only for that test process. Its real
webviews connected to Vite and the Python bridge. Selecting Summarize in the separate native
pie completed the pipeline; selecting Draft a reply and rejecting in the main window
recorded cancellation with no result. Both audit records were checked in an isolated log.

The driver reported pointer synthesis unsupported on this Wayland session. The native
smoke test therefore dispatched DOM clicks in the actual webviews; it did not bypass the
UI or call the bridge directly. Native screenshots of the main window, pie and approval
panel were inspected and saved to ignored `.bloom/previews/`. A subsequent attempt to
recapture screenshots timed out in the driver; interaction-only native validation passed
again after the Tauri JavaScript/Rust versions were aligned. Screenshot automation is not
treated as a reliable unattended check on this desktop session.

The default entry point is now Tauri (`make ui` / `make desktop` / `npm run desktop` from
`shell/`), with Vite hot reload. Browser mode requires `make ui-preview`. Lifecycle tests
cover normal and failed desktop-process exits, and the frontend tests also verify that
preferences synchronize across the main and pie surfaces. Native compilation is included
in CI as a separate job. macOS behavior, explicit external-app focus restoration and
packaged distribution remain open. The overlay follow-up below adds the X11 shortcut.

## Cursor overlay follow-up

Decision: [ADR 0008](../adr/0008-cursor-overlay.md). The pie now uses a 480 × 480 transparent,
borderless, shadowless native webview, without a rectangular panel, title or taskbar entry.
The existing ActionPlan and choice transport are unchanged.

- `make check`: all 51 Python tests, types, lint and architecture contracts passed.
- Frontend formatting, generated contracts, TypeScript and production build passed.
- All seven browser integration tests passed. The separate-surface test now verifies
  transparent HTML/body/panel backgrounds, no header, square radial geometry, Escape and
  transparent-background cancellation, together with the audited Python results.
- `cargo test --locked --offline`: the native geometry test passed, covering centred placement,
  screen-edge clamping, monitors with negative coordinates and scaled dimensions. CI now runs
  `cargo test --locked` rather than only checking compilation.
- `cargo build --locked --offline`: the actual native app compiled.
- Native Wayland WebKit smoke: transparent page, no frame, 480-pixel pie, summary selection,
  approval rejection and both audit records verified. Preferences reported the Wayland limit.
- Native XWayland smoke (`GDK_BACKEND=x11`): the OS shortcut registered. A synthetic Ctrl+Space
  press with an unrelated X11 window focused triggered the actual run. At cursor (800, 550),
  the native overlay's reported origin was (560, 310), centring the 480-pixel menu exactly.
  Selection and approval rejection again produced the expected audit records. Keyboard input
  used XTest; action buttons used DOM clicks inside the actual native webviews.

The native pie screenshot was inspected and saved to ignored `.bloom/previews/native-pie.png`.
Additional focus-loss/keyboard-activation checks could not be completed meaningfully: GNOME's
`org.gnome.ScreenSaver.GetActive` returned true, and its window manager refused to activate
the overlay while locked. These OS focus interactions need verification in an unlocked
desktop session. An attempted additional normal-launch keyboard check did not run because
automatic permission approval review timed out. The successful native checks above do not
depend on that additional check. Browser cancellation checks and the native selection,
shortcut registration/delivery, geometry and approval checks passed.

The X11 test helper initially attempted to focus its window before the compositor mapped it;
mapping the helper before launching the app fixed the test setup. No application change was
needed for that test failure.

Native Wayland cannot use the standard global-hotkey backend or retrieve the global cursor.
Pinned Tao returns a placeholder (0, 0), so the native code detects Wayland explicitly and
uses compositor-controlled placement. This GNOME desktop exposes no GlobalShortcuts portal.
Full native Wayland integration and macOS validation remain open; XWayland shortcut delivery
while a native Wayland app has focus depends on the compositor.

## Main-window-independent capture and background launch

Decision: [ADR 0009](../adr/0009-desktop-cursor.md).

The Linux native reader now queries the X11 root directly. The React receiver no longer
overwrites the anchor captured by the global shortcut. Each native bridge view holds a Web
Lock while mounted, keeping it eligible for background work when the main window is minimized.

- `make check`: passed, including 51 Python tests and all architecture contracts.
- Frontend formatting, TypeScript, generated contracts and production build: passed.
- Eight browser integration tests passed. The new regression uses a mocked native event
  transport with the actual Python bridge and a separate pie page. It verifies no duplicate
  capture on a native trigger, a fresh capture on a button launch, actual audited outcomes,
  a held background Web Lock and release of that lock when the view navigates away.
- Rust geometry test and native build passed with the direct x11rb dependency.
- Native WebKit/XWayland smoke passed: actual global shortcut from a temporary external X11
  window, correct cursor-centred origin, summary selection and approval rejection. The
  driver then minimized the main window; another actual shortcut opened the pie and selecting
  Summarize completed a third audited run without first bringing the main window forward.

The native check covers X11 input surfaces on this GNOME Wayland session. It does not prove
global pointer visibility over native Wayland clients: GNOME can restrict XWayland cursor
coordinates. A compositor-side adapter is still needed for that case. The shared Tauri UI
and Python contracts do not depend on that adapter.

## Optional GNOME Wayland adapter

Decision: [ADR 0010](../adr/0010-optional-gnome-adapter.md).

The GNOME 46 extension supplies the actual Shell pointer and Ctrl+Space through a local
session-bus service. The Linux native adapter uses it when available, while the shared
Tauri UI and Python pipeline retain their existing contracts. The main window also exposes
a native Minimize button to keep Bloom running in the background.

- `make check`: passed, including all 51 Python tests and architecture contracts.
- Frontend formatting, generated contracts, TypeScript and production build: passed.
- All eight browser integration tests passed, including the Minimize command and the
  existing background-trigger regression against the real Python bridge.
- Five Node policy tests passed: capture at activation, idempotent enable, explicit release,
  disconnect cleanup, competing owners and failed shortcut registration.
- `cargo test --locked --offline`: both coordinate/overlay geometry tests passed. The
  separate session-bus integration is intentionally ignored in this ordinary unit run.
- `cargo build --locked --offline`: the final native executable compiled successfully.
- The Rust/GJS private-session-bus test passed using the production D-Bus service. It checks
  version negotiation, pointer reads, trigger delivery, explicit disable, owner isolation,
  release on connection loss, reacquisition and nonblocking shutdown during a GUI callback.
- Native WebKit integration passed with that service on a private bus and simulated Shell
  capture/keybinding hooks. A stage point of (1740, 900) in 1920 × 1080 mapped to the current
  XWayland root; the actual native pie origin was (4980, 1560). Selecting Summarize completed
  the real Python pipeline and its isolated audit record. After clicking Minimize, another
  D-Bus trigger opened the separate pie and completed a second audited run. An external
  X11 window-state query verified that the main window remained minimized.
- The extension archive was built and installed for the current user. GNOME 46.10 has not
  discovered it in this running session; the installer correctly reported the required
  one-time logout/login and enable command.

The private-bus integration does not exercise GNOME's real accelerator hooks or pointer
capture. Those require activation after logout/login and an unlocked desktop. Mixed-scale
monitor validation and other GNOME versions remain open. No desktop session was restarted
or logged out by the installer or tests.

## Live GNOME activation and native window controls

Follow-up to the user's unchanged placement/background behavior and `The name is not
activatable` message. The actual session reported `disable-user-extensions=true`, with Bloom
INITIALIZED and disabled. Installing the adapter had not activated the Shell service.
Extension support was enabled and Bloom activated; GNOME then reported ACTIVE, protocol 1,
and actual desktop pointer coordinates. No new logout was needed in this session.

The GNOME launch target now checks the service before building or starting, and fails with
specific instructions for disabled user extensions, missing discovery or a failed service.
The installer also verifies its endpoint after enabling instead of treating the CLI enable
exit status as proof. Four additional Node regressions cover these health checks.

The in-page Minimize button and command were removed. GTK's standard titlebar now contains
Minimize, Maximize and Close through process-local client-side decorations; see
[ADR 0011](../adr/0011-native-linux-window-controls.md). This does not change GNOME's global
window-button setting. The actual native titlebar screenshot was inspected and is retained
in ignored `.bloom/previews/native-titlebar.png`.

Validation on the active GNOME 46 Wayland desktop:

- `make check`: all 51 Python tests, formatting, lint, types and four architecture contracts
  passed with local server sockets permitted.
- Frontend formatting, contracts, TypeScript, production build and eight browser tests passed.
- Nine GNOME Node tests passed, including the globally-disabled-extension regression.
- Both native Rust geometry tests and the final native executable build passed.
- The live native WebKit run connected to the actual Shell extension on the normal session
  bus. Input used a temporary Mutter remote-input session, closed immediately after each
  operation, to exercise the real Ctrl+Space binding; no fixture signal or mocked Shell hook
  was used. XWayland's synthetic key events did not activate the compositor binding and are
  not treated as an equivalent test.
- At cursor (5000, 1100), outside the main window's geometry, the separate native 480-pixel
  pie opened at (4760, 860), exactly centered. Selecting Summarize completed the real Python
  pipeline with a successful isolated audit record.
- A compositor pointer click on the actual GTK titlebar Minimize control minimized Bloom.
  Another real Shell shortcut opened the pie, and selection completed a second successful
  audited run. An external X11 window-state query verified that the main window stayed
  minimized throughout that second selection.

The shortcut test focused an unrelated X11 helper and moved the cursor outside Bloom. Native
Wayland-client focus and mixed-scale monitor behavior remain separate validation items.
