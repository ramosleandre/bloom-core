# 05 · Testing

The structure exists to make most of Sense testable without a desktop.

## Layers

| Layer | What runs | Where |
|---|---|---|
| 1 · Core, pure | reducer, precedence, privacy filter, timeline bounds, API handlers, with scripted sources from `sense-fake` | every machine, CI, all three OSes |
| 2 · Replay | recorded source streams (`fixtures/*.jsonl`) fed through the real core; the resulting state and snapshots are compared to expected files | every machine, CI |
| 3 · Conformance | a real source against a real desktop, driven by a fixture application | per platform, see below |
| 4 · Cross-compile | `cargo check` for the macOS, Windows and Linux targets | CI, on every pull request |

Layers 1, 2 and 4 are required checks. They catch nearly every regression in shared code and every
compile error in platform code, from any developer's machine.

## The conformance suite

One set of scenarios, the same for every platform. Each scenario is: put the fixture application
in a known situation, then assert on what the engine reports.

The **fixture application** is a tiny program per toolkit family (a native window with a text
field, a multi-line text, a password field, a document title; plus a static web page for browser
scenarios). Its text is fixed, so assertions are exact.

Scenarios (initial):

1. Fixture comes to front → `focus.app`, `focus.window`, title, within 200 ms.
2. Select a known sentence → `selection.text` equals it, with the expected `quality`.
3. Focus the password field, type → `secure` is true, no content anywhere, including `inspect`.
4. Copy text → `clipboard.changed`, kinds, owner; concealed copy → no content.
5. `content.read` Surrounding / Visible / Document → expected text; deadline of 1 ms → `Timeout`,
   returned on time.
6. Open a client window registered in `own_pids` → `focus` unchanged.
7. Revoke the permission or stop the helper → `capabilities` changes, values become `Unavailable`
   with the right reason, no hang, no crash.
8. Excluded application in front → category only, no title, no content.
9. Lock the session → state cleared, only `activity.*` events.
10. One hour of synthetic events → memory stays within the bound.
11. At level 1, use the fixture for a minute, then query → timeline empty, state holds focus
    identity only; a `capture` still returns title and selection, and they are gone afterwards.
12. Mark the fixture window as excluded from capture (macOS, Windows) → no title, no content, by
    any channel.
13. Fixture web page with the `desktop-context: none` hint → URL and content excluded.
14. Any source running → the indicator is visible; a `capture` makes it signal the client's name.
15. Fixture text containing a test card number and a private-key block → both replaced by typed
    placeholders, `redacted` set; a 16-digit number failing the Luhn check is left alone.
16. A client in "ask per application" mode reads from a new application → `approval_pending` until
    answered; "never" is remembered.
17. After scenarios 2 and 5, `access.log` lists both reads with client, application and sizes, and
    contains none of the fixture text.

Where it runs:

- **Linux `x11`**: automatically in CI, under a virtual X server. This keeps the Linux crate honest
  on every pull request.
- **Linux `gnome`**: in a nested or headless GNOME Shell session where CI allows; otherwise by the
  owner before merging a change to that source.
- **macOS, Windows, `kde`, `wlroots`**: by the platform's owner before merging a change to that
  platform's sources; results are noted in the pull request.

## Recording and fixtures

`sense record out.jsonl` writes the observation stream *as sources emitted it*, anonymized at write
time (`04-privacy-security.md`). A recording plus the expected state is a replay test. Each new
source and each bug fixed in a source adds one.

Recordings from different machines and desktops are the cheap way to cover the matrix: a developer
on one OS runs the replay of every other OS's recordings.

## Budgets asserted in tests

| What | Budget |
|---|---|
| `state.get` | under 2 ms |
| `capture` with no content | returns within its deadline, always |
| daemon idle CPU, ambient tier, no user activity | under 0.5 % of one core |
| daemon memory, steady state | under 80 MB |
| ambient event to state update | under 20 ms |

These are design targets. They become measured thresholds once the first sources exist.

## Lints

- Forbidden API list per platform file, checked by a script over the platform crates.
- No networking crate in the dependency tree of the daemon (`cargo deny` rule).
- No `cfg(target_os)` in `sense-core` or `sense-schema`.
- Log output of the replay suite contains no fixture content.
