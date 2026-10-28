# Regressions caught (O2)

> At least 3 regressions caught by tests, documented before end of March; at least one case where CI blocked a broken PR before `main`.
> The guide: *"A test that has never caught anything proves nothing; a test that stopped a broken release proves everything."*

## Regressions caught by a test

| Date | What broke | Test that caught it | PR | Fix |
|---|---|---|---|---|
| 2026-10-23 | Pressing Escape immediately after opening a pie could leave it visible while the scheduled focus callback had not run | Playwright keyboard navigation and Escape test | Local development | Set initial focus in a layout effect and capture Escape at the document while the dialog is mounted |
| 2026-10-23 | React recaptured the cursor after the native shortcut had already stored the invocation position | Background native-trigger integration test against the real Python bridge | Local development | Mark native triggers as already prepared; keep fresh capture for button launches |
| 2026-10-23 | Concurrent D-Bus signal polling could consume a synchronous shortcut-registration reply, causing a timeout | Private-bus Rust ↔ GJS session contract test | Local development | Serialize method calls and signal dispatch on one worker thread; verify registration, activation and client-disconnect cleanup |
| 2026-10-23 | Installing/enabling the extension could appear successful while GNOME's master switch still disabled all user extensions, leaving Bloom on the limited fallback | GNOME launch-check regression tests, reproducing the user's `The name is not activatable` report | Local development | Probe the live service before the GNOME launch/build and report the master switch, missing service or failed extension explicitly |

## PRs blocked by CI

| Date | PR | CI step | What was broken |
|---|---|---|---|

## Integration issues caught during validation

| Date | Check | Issue | Fix |
|---|---|---|---|
| 2026-10-23 | Native `make ui` launch | The JavaScript API had resolved to Tauri 2.12 while Rust 1.86 selected the compatible Tauri 2.11 crate; the CLI reported a minor-version mismatch | Pin the Rust crate, JavaScript API and CLI to the 2.11 release line; both native webviews and the UI checks were revalidated |
| 2026-10-23 | Actual GNOME session service probe | `disable-user-extensions` was true and Bloom was INITIALIZED/disabled, so no session-bus service existed | Turn on extension support and enable Bloom; verify ACTIVE, protocol 1 and actual global pointer reads before native UI validation |
