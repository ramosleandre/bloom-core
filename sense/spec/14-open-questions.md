# 14 · Open questions and how to close them

What is still unknown, how sure we are, and the cheapest way to find out. Update this file as
answers arrive; when a question is closed, move its conclusion into the relevant spec and delete
it here.

Confidence: **high** = confirmed by a primary source or by widely deployed precedent; **medium** =
precedent exists but not for our exact use; **low** = reasoning only.

## Closed by desk research (November 2026)

| Question | Answer | Source |
|---|---|---|
| Can a GNOME Shell extension read the focused window's title and class, and the pointer? | Yes. `Meta.Window.get_title()`, `get_wm_class()`, `global.get_pointer()`. The pointer needs version-specific fallbacks on recent Shells. | Kando's integration extension (MIT), in production across Shell versions |
| Can it read the PRIMARY selection? | The API exists, but the channel is dead by default: since GNOME 50 primary paste is disabled by default and GTK applications publish no primary selection. The specs no longer depend on it. | GNOME `gsettings-desktop-schemas` change; Debian changelog; our spike |
| Can it inject a paste shortcut? | Yes: a Clutter virtual keyboard device and `notify_key`. | Kando's `InputManipulator.js` |
| Can it grab a global accelerator? | Yes: `global.display.grab_accelerator`. | Kando's `Shortcuts.js` |
| Does Mutter offer data-control, which would make the extension unnecessary for the clipboard? | No, as of Mutter 51. KWin, Sway, Hyprland, niri, COSMIC, labwc and river do. | Wayland protocol support tables |
| Will the extensions site accept clipboard access? | Yes, with conditions: declared in the description, no sharing with a third party without explicit user interaction, no default clipboard shortcut, no bundled binary. Our on-request design satisfies them. | extensions.gnome.org review guidelines |
| Does a child process inherit its parent application's TCC grant on macOS? | In the ordinary case yes, per Apple's developer support, tested on macOS 15. Failures were reported with a helper packaged as a nested application bundle. | Apple developer forums |
| Do accepted assistants read on request and ask per application? | Yes: Claude's computer use asks per application and has a blocklist; ChatGPT reads the front application when prompted; Recall, the continuous one, had to add opt-in and a tray indicator. | vendor documentation |

## Closed by our own spike (`sense/spike/gnome-shell/`, GNOME Shell 50.6, headless, one toolkit)

These show the mechanism works. They are one environment, not a reliability measurement.

| Question | Answer |
|---|---|
| S1 · Extension connects out to a Unix socket, asynchronously, and reconnects | Yes. About 0.5 ms per request and reply; back 0.3 s after the server dropped the connection. The "extension exports nothing" design stands. |
| Focused window read from inside the Shell | Yes: title, class, process id, about 0.3 ms. |
| A window activated by the extension receives keyboard focus | Yes for a GTK 4 window. Block D's prototype shows its XWayland overlay receiving input on GNOME 46, which covers most of S7. |
| Paste through the Shell's virtual keyboard | Yes in a single trial with a 50 ms delay. Reliability across applications and after our own window held focus is still S3. |
| Reading another application's clipboard from the extension | Yes, about 1 ms. This makes the copy-based selection read viable on GNOME. |
| S2 · PRIMARY as the selection channel on GNOME | **No.** No owner by default on GNOME 50; with the setting re-enabled, an owner appeared but the read returned an empty string. Design changed: `platform-linux.md`, `12-acting.md`. |

## Open: needs a prototype (the spike)

One week, throwaway code, one page of results in `lab/results/`. Each question has a pass
criterion and a consequence, so the result changes a decision.

| # | Question | Confidence now | How to test | If it fails |
|---|---|---|---|---|
| S2b | Copy-based selection read: how often does it work across applications, how long does the clipboard take to change, and which applications need a different copy shortcut? | medium (worked once in the spike) | 10 applications × 10 trials: browser, editor, terminal, mail, office, chat, files. Record success, delay, and side effects. | Selection-based actions are offered only where accessibility or an extension provides the selection. |
| S3 | After our own window had focus, does "activate target window, then inject the paste shortcut" land the paste in the target reliably, and what delay is needed between the two? | medium-high (worked once in the spike without the focus round trip; Kando does it in production) | Scripted: 50 repetitions into three applications; count failures at 0, 50, 100 ms delay. | Use the RemoteDesktop portal for keys (consent dialog once), or copy to the clipboard and tell the user to paste. |
| S4 | macOS: with `sensed` as a plain executable inside the client's bundle, launched by the client, does one Accessibility grant to the client cover it? In a development build? After an update? | medium-high | Minimal Tauri app + helper calling `AXIsProcessTrusted` and reading the focused element; test `cargo tauri dev`, a signed bundle, and a re-signed update. | Make the helper its own signed agent with its own grant (one extra line in onboarding). |
| S5 | Does a level-1 `capture` (nothing cached: compositor helper + accessibility + browser extension round trips) fit in 50 ms at p95? | low | Stub sources with the real transports (socket to extension, native messaging to browser, one AT-SPI call); measure 200 captures. | Options, in order: show the menu first and fill slots when the capture lands; keep the *focused element's* descriptor warm at level 1 (metadata only, a small change to the level definition); raise the budget. |
| S6 | Is matching a browser tab to the OS window by "same application focused + title agrees" reliable? | low-medium | 30 switches across two browser windows and two browsers. | Have the extension report the browser's own window title or geometry; or accept `ambient` quality for `document.url` when the match is weak. |
| S7 | Does a Tauri/WebKitGTK window moved and raised by the extension on GNOME receive keyboard focus? | medium | With S3. | Menu is pointer-only on GNOME, or drawn by the extension itself (large change; avoid). |

S2b, S3 and S7 need a GNOME machine with real applications; S4 needs the team's Mac; S5 and S6
need either. The headless, isolated Shell session used by the first spike (`run.sh`) avoids
re-logging at each change and can be reused for the scripted parts. Repeat the closed checks on
GNOME 46 (Ubuntu 24.04) before relying on them there.

Stage 0 of the work packages does not depend on any of these and can start in parallel.

## Open: needs a decision by the team

| # | Decision | Recommendation | What it blocks |
|---|---|---|---|
| D1 | Accept the direction: a separate daemon in Rust (ADR 0012) | Accept. The alternative keeps OS code in Python and leaves the macOS permission and Wayland problems where they are. | everything |
| D2 | On request by default, ambient opt-in (0013); privacy model (0016) | Accept; they are what makes the product defensible in front of a jury or a user. | stage 0.3 |
| D3 | Linux by session family, own Shell extension (0015) | Accept; ADR 0007's X11 target cannot run on current GNOME. | stage 1 |
| D4 | Acting service with closed effects (0018) | Accept. | block C's `execute`, block D's hotkey |
| D5 | MCP server and public profile (0017) | Defer. Nothing depends on it before stage 6. | nothing |
| D6 | The `Observation` contract change (`13`) | Review as a `contract` pull request once D1 is taken. | Bloom integration |
| D7 | Ownership: who owns `sense/`, who runs macOS conformance, who reviews the extension | Block C owns `sense/`; the Mac's owner runs macOS conformance; two reviewers on anything that injects input. | stage 1 |
| D9 | One GNOME extension or two: block D's `bloom-desktop` (pointer, hotkey, over D-Bus) and Sense's (window, clipboard, paste, over an outbound socket) | Keep both until the slice works; then merge into one extension that keeps D's hotkey ownership rule and uses the outbound socket for anything that returns content or injects input. | nothing now; install friction later |
| D8 | Name | Keep "Sense" as a working name until publication is considered. | nothing |

### If the team refuses a Rust daemon

A reduced variant keeps most of the value: the same schema, levels, privacy filter and source
contract implemented in Python inside the surface block, with `linux.gnome` (the extension is
JavaScript either way), `linux.x11`, macOS through `pyobjc`, and the browser extension. What is
lost: the single stable identity for macOS permissions (the open risk of ADR 0004 returns), the
MCP server for other assistants, and independence from the Python daemon's lifetime. The specs
are written so that the schema, the extension protocol and the conformance scenarios carry over
unchanged.

## Open: needs someone outside the team

| # | Question | Who | Why it matters |
|---|---|---|---|
| L1 | When Bloom sends context to a hosted decision or generation model, who is the controller, what is the legal basis, and is a processor agreement with the model provider in place and adequate? | the school's legal or data-protection contact; the mentor to point to them | The engine is local; this transfer is the project's actual GDPR exposure. |
| L2 | Context routinely contains other people's personal data (an email's sender and text). Is "minimize, label, ask approval before sending" sufficient for a student project with test users? | same | Third parties never consented to anything. |
| L3 | If outside testers are recruited: what information notice and consent form are needed? | same | Required before the reproducibility test by someone outside the team. |
| L4 | Does publishing a GNOME extension and a browser extension under the team's names create obligations (privacy policy URL, data-use declarations)? | store policies; answerable by reading them at submission time | Both stores require declarations at upload. |

Prepared for that conversation: the data-flow in one paragraph. *Sense reads context on the user's
machine at the user's request and keeps it in memory. Bloom's context block labels each piece by
sensitivity. Bloom's policy sends to a hosted model only what the labels allow, and shows the user
the real content before anything leaves the machine through an action. Nothing is stored by the
team; there is no server of ours.* If any part of that sentence is false in the shipped product,
that part is the problem to fix.

## Facts that rest on secondary sources

Re-verify before quoting outside the team: the release status of native MCP in Windows; the
WebMCP timeline; benchmark figures for hybrid perception (reported by the systems' authors); the
details of ChatGPT's "Work with Apps" (OpenAI's help page could not be fetched; taken from press
coverage); Perplexity's desktop product (not checked at all).
