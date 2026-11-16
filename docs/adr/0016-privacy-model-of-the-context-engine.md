# 0016 · Privacy model of the context engine

- **Date**: 2026-11-16
- **Status**: proposed
- **Deciders**: Everyone

## Context and problem

[ADR 0013](0013-read-on-request-ambient-opt-in.md) puts a resident process on the user's machine that can read
windows, selections and the clipboard. That is the technical shape of spyware. The
difference must be structural and verifiable, not a promise in a README. It also conditions the
project's privacy criterion and its GDPR position.

## Options considered

1. Collect broadly and let each client filter.
2. Ask the user to approve every read.
3. Constrain the engine itself: local only, memory only, metadata by default and content on
   request, one filter that every observation passes, user-visible inspection and pause.

## Chosen option

3. Details: [`sense/spec/04-privacy-security.md`](../../sense/spec/04-privacy-security.md). The
rules are derived from the accepted definition of stalkerware (remote monitoring of another
person's device, without consent, without persistent notification) and from how comparable
products were received:
[`sense/spec/07-prior-art-and-acceptability.md`](../../sense/spec/07-prior-art-and-acceptability.md).

- The observed person is the person in control: no remote viewer, no account, no multi-user or
  administrator-managed observation mode, no stealth option.
- Nothing is observed before a consent screen; the default level reads content only on request
  ([ADR 0013](0013-read-on-request-ambient-opt-in.md)).
- A visible indicator is present whenever a source runs; it signals each content read and offers
  pause. It cannot be hidden.
- Existing opt-outs are honoured on every channel, accessibility included: windows that refuse
  screen capture, secure fields, concealed clipboard entries, private browsing, and a provider
  hint by which an application or web page can decline.
- No audio.
- Uninstalling removes every registered component.
- The daemon contains no network code; CI enforces it on the dependency tree.
- State and timeline live in bounded memory and are erased on lock, pause and exit.
- Never collected: keystrokes, secure fields, concealed clipboard entries, private browsing
  windows, excluded applications (password managers by default), background screenshots.
- A single privacy filter in the core applies these rules before storage; sources and clients
  cannot bypass it.
- Every value is labelled `system`, `personal` or `content`; `content` is third-party, untrusted
  input.
- The local socket is user-only; readers get scopes per executable; sources authenticate with a
  token and may only emit what they declared.
- Only official OS interfaces.

## Rationale

- Option 1 puts the guarantee in every client, where it will eventually be missed.
- Option 2 makes the product unusable and trains the user to click through.
- With option 3 the guarantees are properties of one component and can be tested: no network
  dependency, table-driven filter tests, a conformance scenario for password fields, a check that
  logs contain no content.

## Accepted consequences

- Some context is deliberately lost (private windows, password managers, some clipboard entries,
  every application that protects its windows from capture, such as messengers and banking apps).
- The indicator, the consent screens, the inspector and the uninstaller are product features to
  build before any outside user receives a build.
- Sense cannot be sold or adapted as a monitoring tool for employers or parents.
- What a client sends to a hosted model is outside the engine's control: GDPR obligations for that
  transfer (legal basis, processor agreement, third parties' data) sit with the client and need a
  proper review. This ADR is not legal advice.
- Honest limit: on macOS and Windows, any program running as the user can call the same OS APIs
  itself, so Sense adds little attack surface there. On Wayland it does add some, and its socket
  is the boundary to defend.
- Long-term storage of context is out of the engine; a client that stores it carries its own
  retention and consent obligations.
