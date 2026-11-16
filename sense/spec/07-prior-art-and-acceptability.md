# 07 · Prior art and acceptability

This file answers two questions. How do existing products get the user's context, and how were they
received? And what must Sense do, concretely, to stay on the acceptable side? Its conclusions are
normative: they changed the default behaviour of the engine (see "What this review changed").

Sources are listed at the end. Product facts are as reported publicly up to November 2026; check
them again before quoting them outside the team.

## How others do it

Three families exist. They were received very differently.

### 1 · Read when the user asks

| Product | Mechanism | Notes |
|---|---|---|
| ChatGPT desktop, "Work with Apps" (macOS) | The macOS accessibility API reads the front window of supported apps when the user sends a prompt; VS Code needs a companion extension. | The app being read is shown in the prompt box. Turned off by a switch or by revoking the Accessibility permission. |
| Apple Intelligence, onscreen awareness | Applications **donate** what is on screen: they describe it as entities (App Intents, `NSUserActivity`). The assistant does not scrape other apps. | The application's developer decides what is exposed. Apple platforms only. |
| Android Assist API | At invocation, the OS hands the assistant a structured view of the current screen. Applications can refuse (`FLAG_SECURE`) or add richer content. | An OS-level contract with an application opt-out, in place for a decade. |
| Claude desktop, computer use | Screenshots of the real desktop plus mouse and keyboard control, during a task the user started. | No sandbox between the assistant and applications. Approval is asked per application; a blocklist exists and trading and cryptocurrency apps are blocked by default. |
| Highlight and similar overlays | Screenshot plus text recognition when the assistant is opened. | |

Reading at the user's request, with a visible sign of what is read, is the accepted norm. Nobody
calls these products spyware.

### 2 · Record continuously

| Product | Mechanism | What happened |
|---|---|---|
| Microsoft Recall | Periodic screenshots, text recognition, a searchable local history. | Announced on by default with plain local storage; withdrawn after a public backlash; reissued as opt-in, removable, encrypted, gated by user presence, with filters for apps, sites, private browsing and detected sensitive data, and a system-tray icon whenever snapshots are being saved. Some applications (Signal) still block it by marking their windows as protected content. |
| Rewind, then Limitless | Continuous screen and audio capture on the Mac. | Capture was switched off after the company was acquired in December 2025. Users' histories depended on the vendor's fate. |
| screenpipe | Open source, continuous screen and audio capture to a local database, exposed to assistants over MCP. | Several gigabytes of disk per month. A deliberate opt-in product for people who want exactly that. |
| Littlebird and similar | Continuous reading of on-screen text, stored in the vendor's cloud. | |

Continuous capture of content is where the controversy is. The lessons are consistent: opt-in only,
never by default; users and third-party applications demand ways to exclude themselves; a stored
history is a liability (theft, vendor change, legal discovery).

### 3 · Track activity metadata

ActivityWatch (open source), RescueTime, WakaTime: small watchers report the active application,
window title, browser tab and idle state to a local server. No content. This class has existed for
years without scandal when it is local and self-installed. ActivityWatch's architecture (watchers
feeding one local server, one typed stream per watcher) is the same shape as Sense's sources.

## Where the line is

The Coalition Against Stalkerware defines stalkerware as software that lets a **remote** person
monitor the activity on **another** person's device **without consent** and **without explicit,
persistent notification**. App-store policies add: data leaving the device without adequate notice,
no clearly identifiable icon, hard to remove. Each element of those definitions becomes a
requirement:

| Spyware trait | Sense requirement |
|---|---|
| Someone else watches | The observed person is the person who installed and controls Sense. There is no remote viewer, no account, no network code, no multi-user or administrator-managed observation mode. |
| No consent | Nothing is observed before an onboarding screen that says in plain words what each level reads. Each OS permission and each extension is a separate, explained step. |
| No persistent notification | A visible indicator (menu bar or tray) is present whenever a source is running. It shows the current level and signals each content read. No setting hides it. |
| Hidden presence | Plain process and extension names, listed in login items and in each host's extension list. No stealth option. |
| Hard to stop or remove | Pause is one click from the indicator. Uninstalling removes the daemon, the extensions' registration and the native-messaging manifests. |
| Collects more than the purpose needs | Default level reads content only at the user's request. Keystrokes, secure fields and background pixels are never collected at any level. |
| Data leaves the device | It cannot leave through Sense. A client that sends context to a remote model owns that decision and must show it to the user. |

## Honour every existing opt-out

Applications and people already have ways to say "do not capture this". Sense treats each of them
as an exclusion for **every** channel, including accessibility reads, not only for screenshots:

| Signal | Set by | Meaning for Sense |
|---|---|---|
| Window excluded from capture (macOS window sharing type "none"; Windows display affinity other than "none") | the application (messengers, banking, password managers, DRM players) | window is `excluded`: no title, no content, by any channel |
| Secure text field, password role, HTML password and credential fields | the application or page | never read |
| Concealed and transient clipboard markers | password managers | event without content |
| Private or incognito browsing | the user | not observed |
| Session locked | the user | nothing observed |
| Provider hint `context: none` (ours, see `08-open-protocol.md`) | any application or web page | excluded |

The first row goes further than Recall did, and answers the objection that led Signal to block it:
an application that already refuses screen capture is refusing to be read, whatever the mechanism.

## Observation levels

The acceptable default follows from the prior art: family 1 by default, family 3 as an explicit
option, family 2 never.

| Level | Name | What runs | Kept in memory | Comparable to |
|---|---|---|---|---|
| 0 | Paused | nothing | nothing | |
| 1 | **On request** (default) | Only what is needed to know which application and window are in front, plus idle and lock state. Everything else is read when the user invokes a client. | The current focus identity. No titles, no history. | ChatGPT Work with Apps, Android Assist |
| 2 | Session awareness (opt-in) | Ambient metadata sources: titles, tabs, selection and clipboard *events*, files, commands, media. | A bounded timeline of metadata, in RAM, erased on lock and exit. | ActivityWatch |

Content in depth (document text, accessibility subtree, pixels) is read only on request at every
level. No level writes history to disk or captures pixels in the background. Moving to level 2 is a
separate consent screen that lists what is added.

## Rules outside the code

Not legal advice; the team should have these points checked for the final product.

- **GDPR.** While everything stays on the user's machine for their own use, the team receives no
  personal data. The moment a client sends context to a hosted model, that transfer is processing
  with a controller, a legal basis, a processor agreement and a location: that analysis belongs to
  the client (Bloom), and the `content` and `personal` labels exist to support it. Context often
  contains **other people's** data (the sender of an email, a chat partner): minimize what is sent.
- **Workplace.** Sense must not be usable as an employee-monitoring tool: no reporting, no remote
  configuration of observation. Say so in the README and the licence notice.
- **Audio.** Out of scope entirely. Recording conversations triggers consent laws that vary by
  country.
- **Distribution policies.** Chrome Web Store (single purpose, minimal permissions, limited use of
  user data, prominent disclosure, no remote code); Firefox Add-ons (declared data collection and
  consent); GNOME extensions review (clipboard access is scrutinized, no telemetry); macOS
  notarization outside the Mac App Store (the App Sandbox required there is incompatible with the
  accessibility API). Each extension's listing must describe exactly what it reads.
- **Security software.** Clipboard listeners, accessibility clients and window hooks match the
  heuristics of antivirus products. Signed binaries, no obfuscation and public source reduce false
  positives; expect to file some anyway.

## What this review changed in the design

1. The default is level 1, on request. The first draft had ambient observation always on.
2. A persistent, non-hideable indicator is a requirement, not an optional feature.
3. Windows excluded from screen capture are excluded from every channel.
4. No remote, multi-user or managed observation mode will exist.
5. Uninstall must remove every registered component.
6. Sense is offered to other assistants through an existing standard (MCP), off by default. Its
   schema and rules of conduct may be published as a small profile on MCP, not as a new protocol
   (`08-open-protocol.md`).

## Sources

- Recall's redesign: [Tom's Hardware](https://www.tomshardware.com/software/windows/microsoft-describes-recalls-new-security-features-says-the-feature-is-opt-in), [SecurityWeek](https://www.securityweek.com/microsofts-controversial-recall-returns-with-proof-of-presence-encryption-data-isolation-opt-in-model/amp/); Signal's countermeasure: [CyberInsider](https://cyberinsider.com/signal-deploys-countermeasure-to-shield-messages-from-windows-recall/)
- Claude computer use: [Anthropic support](https://support.claude.com/en/articles/14128542-let-claude-use-your-computer); Recall safeguards in Microsoft's words: [Windows blog](https://blogs.windows.com/windowsexperience/2024/09/27/update-on-recall-security-and-privacy-architecture/)
- ChatGPT Work with Apps: [OpenAI help](https://help.openai.com/en/articles/10119604), [Gigazine](https://www.gigazine.net/gsc_news/en/20241115-chatgpt-macos)
- Apple onscreen awareness: [Apple, App Intents updates](https://developer.apple.com/documentation/updates/appintents.md), [Superwall walkthrough](https://superwall.com/blog/how-to-integrate-siri-and-apple-intelligence-into-your-app-to-query-onscreen)
- Rewind and Limitless: [9to5Mac](https://9to5mac.com/2025/12/05/rewind-limitless-meta-acquisition/); screenpipe: [project site](https://screenpi.pe/about); Littlebird: [TechCrunch](https://techcrunch.com/2026/03/23/littlebird-raises-11m-to-capture-context-from-your-computer-so-you-can-query-your-data/)
- ActivityWatch: [architecture](https://docs.activitywatch.net/en/latest/architecture.html), [data model](https://docs.activitywatch.net/en/latest/buckets-and-events.html)
- Stalkerware definition: [EFF on the Coalition Against Stalkerware](https://www.eff.org/press/releases/eff-antivirus-companies-and-human-rights-groups-launch-coalition-combat-stalkerware); Google Play monitoring rules: [The Register](https://www.theregister.com/2020/09/17/google_bans_stalkerware_android_apps/)
