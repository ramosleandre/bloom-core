# 08 · Standards: what to adopt, what to publish

Two questions. Is there an existing standard for "what the user is doing on their desktop, for an
assistant"? And is it reasonable for us to create one (working name: Desktop Context Protocol, DCP)?

Short answers:

- **No standard covers it**, but the ground is moving fast, and it is moving toward the Model
  Context Protocol (MCP).
- **A new protocol is not reasonable.** A small, open **profile on top of MCP** is: a schema, a set
  of naming conventions, and consent rules. That is what "DCP" should mean, if the name is kept.

## What exists (November 2026)

### Delivering context to an assistant

| Standard | Covers | Status |
|---|---|---|
| **MCP** | How an AI application obtains context and tools from a server: resources, change notifications, tools, over JSON-RPC 2.0. | Vendor-neutral governance (Agentic AI Foundation, under the Linux Foundation). Supported by the major assistants. Being built into operating systems (below). |

### Applications describing themselves to agents ("donation")

| Mechanism | Platform | Status |
|---|---|---|
| On-device registry of MCP connectors, with packaging, identity and containment | Windows 11 | Preview since late 2025; built-in connectors for File Explorer and Settings |
| AppFunctions: applications expose functions as tools, described as on-device MCP-style servers | Android 16+ | Platform API available; assistant integration in limited preview |
| WebMCP: a web page registers tools for agents | Chrome | Early preview behind a flag; a W3C community-group draft, not on the standards track |
| App Intents, `NSUserActivity` | Apple | Shipping; Apple's own assistant only |
| nothing | Linux desktops | One mailing-list proposal for a system AI interface (2025), not adopted |

### Reading applications from outside ("scrape")

| Interface | Notes |
|---|---|
| Accessibility APIs (macOS AX, Windows UI Automation, Linux AT-SPI), WAI-ARIA | The de facto universal read interface. Three APIs, designed for assistive technology, no consent model for other uses. |
| Open-source desktop MCP servers (Peekaboo, Windows-MCP, linux-desktop-mcp, kwin-mcp, Cua and others) | A crowded field. Each wraps one platform's accessibility tree and input injection for computer-use agents. Each has its own tool names and output shape. None shares a schema, a consent model or an opt-out. |
| Newton (GNOME, in progress) | Wayland-native accessibility in which applications push their trees. Not final. |

### Activity logs

ActivityWatch (active, local, metadata only), Zeitgeist (dormant), W3C Activity Streams (social
feeds; naming inspiration at most).

## Reading the landscape

1. **The consumer protocol is settled.** It is MCP. Inventing another would be wasted work.
2. **The provider protocol is being settled by platform owners, also around MCP.** Applications
   will describe themselves through their OS's registry or through WebMCP. No application author
   will implement a third-party source protocol from a student project. Our source protocol
   (`02-sources.md`) is therefore an **internal** interface for our own extensions, not something
   to standardize.
3. **Three things remain uncovered**, and they are the same three on every platform:
   - a common **shape** for the present situation (focus, selection, document, clipboard, recent
     activity) with provenance and quality. Every desktop MCP server invents its own;
   - **consent and opt-out semantics** for reading other applications: levels, a visible
     indicator, and a way for an application or page to decline. This is the gap applications
     complained about with continuous-capture products;
   - **the Linux desktop**, which has no donation mechanism and no vendor to build one.

## What DCP would be, concretely

Not a protocol. A **profile**: a short document that says how a desktop-context server presents
itself over MCP, so that any assistant can ask the same question on any OS and get the same shape
of answer.

| Part | Content | Size |
|---|---|---|
| 1 · Schema | The state subtrees and the envelope (source, time, quality, sensitivity, explicit `unavailable` reasons), as JSON Schema. Already written for Sense (`01-data-model.md`). | a few pages |
| 2 · MCP conventions | Resource URIs (`desktop://focus`, `desktop://selection`, …), tool names and arguments (`capture_context`, `read_content`), the text rendering for models (`09`), required annotations. | two pages |
| 3 · Conduct | What a conforming server must do: read content only on request by default, show a persistent indicator, honour the listed opt-outs (secure fields, capture-excluded windows, private browsing, the `desktop-context` hint), sanitize text. | two pages |
| 4 · Conformance | The scenarios of `05-testing.md`, runnable against any server. | exists |

What it would do for someone who is not us:

- An assistant author writes "read `desktop://focus`" once and it works against Sense on Linux,
  and against any other server that adopts the profile on macOS or Windows.
- The author of an existing desktop MCP server can adopt the resource shapes without changing
  their internals.
- An application or web author has one documented way to say "do not read me".

### The MCP mapping (reference)

`sense-mcp` is a separate small binary that connects to the daemon as a reader and speaks MCP to
the assistant that launched it.

| MCP | Sense |
|---|---|
| Resources `desktop://focus`, `selection`, `document`, `clipboard`, `editor`, `terminal`, `timeline` | the corresponding state subtrees (JSON as structured content, the card as text) |
| Change notifications (the subscription mechanism of the current MCP revision; it signals that a resource changed, the client re-reads) | `events.subscribe`, coalesced |
| Tool `capture_context` | `capture` |
| Tool `read_content { kind }` | `content.read` on the current target |
| Tool `list_capabilities` | `capabilities.get` |

Safeguards for this door, because an MCP client usually forwards what it reads to a hosted model:
off by default; enabling it names the assistant being connected; approval asked per application
on first read; per-assistant scopes, with pixels
and clipboard text never granted by default; results carry sensitivity labels; the indicator shows
which assistant read what; no actuation tools.

## Is it reasonable and feasible?

| Question | Assessment |
|---|---|
| Effort to write | Small. Parts 1 and 4 exist as by-products of building Sense. Parts 2 and 3 are a few pages. Days, not months. |
| Effort to keep | Real. A published schema is a compatibility promise. This is why publication waits for the schema to survive two platforms and the browser source. |
| Chance of outside adoption | Low, honestly. Platform owners are building their own stacks; open-source server authors have little incentive to converge. A profile adopted by one implementation is documentation. |
| Value without adoption | Still positive: a disciplined schema, a public and reviewable position on consent, an MCP server that makes Sense usable by assistants other than Bloom, and public contributions for the track. |
| Risk | Scope. Standards work is attractive and endless. It must not delay the engine or Bloom's slice. |
| Where it could actually matter | Linux. With no platform owner, a well-behaved, cross-desktop context server with a consent model is a credible contribution, and the GNOME and KDE communities are the audience (Newton, portals). |

Verdict: **do the profile, not a protocol; do it late; spend days on it, not weeks.** Decision
record: ADR 0017.

## The long-term home on Linux: a portal

On Linux the natural place for a standard is not a new protocol but a **freedesktop portal**: a
"context" portal through which an application asks the desktop for the present situation, with the
desktop's own consent dialog and permission store. That would work for sandboxed applications and
would make our compositor helpers unnecessary. No such portal exists and getting one accepted is a
multi-year upstream effort, so it is a direction, not a plan. If Sense works well, its schema and
consent rules are the material for that proposal.

## Sense as a consumer of donation channels (later)

As applications begin to describe themselves, Sense should read those descriptions instead of
scraping the same applications:

- Windows: connectors in the on-device registry, for the application in focus.
- Web: tools and context a page registers through WebMCP, relayed by `ext.browser`.
- Any application that ships an MCP server exposing its current document.

Each is one more source behind the existing `Source` contract (channel `app_extension`), ranked
above accessibility by the precedence table. Nothing in the engine changes. This is not planned
work: the mechanisms are in preview. It is recorded here so the design stays compatible with where
the platforms are heading.

## Publishing plan

- Develop inside this repository until stages 0 to 2 of the work packages are done.
- Then decide, with evidence, whether to move `sense/` to a public repository (code under
  Apache 2.0, the profile under CC BY 4.0, extensions as separately installable artefacts).
- If published, announce where the neighbours are: the MCP community (a server plus a proposed
  convention for desktop context resources), authors of existing desktop MCP servers, and the
  GNOME and KDE accessibility discussions.

## Sources

- MCP governance and roadmap: [Anthropic's announcement](https://www.anthropic.com/news/donating-the-model-context-protocol-and-establishing-of-the-agentic-ai-foundation), [MCP roadmap](https://modelcontextprotocol.io/development/roadmap.md), [specification changelog](https://modelcontextprotocol.io/specification/2026-07-28/changelog)
- Windows: [Windows developer blog, Ignite 2025](https://blogs.windows.com/windowsdeveloper/2025/11/18/ignite-2025-furthering-windows-as-the-premier-platform-for-developers-governed-by-security), [The Register on the preview](https://www.theregister.com/2025/12/08/microsoft_windows_insiders_agentic_future/)
- Android: [AppFunctions overview](https://developer.android.com/ai/appfunctions?hl=en), [9to5Google](https://9to5google.com/2026/02/25/android-appfunctions-gemini/)
- Web: [WebMCP early preview](https://venturebeat.com/infrastructure/google-chrome-ships-webmcp-in-early-preview-turning-every-website-into-a), [W3C community-group notes](https://w3c-cg.github.io/aikr/webMCP/webmcp-technical-notes.html)
- Apple: [App Intents updates](https://developer.apple.com/documentation/updates/appintents.md)
- Desktop MCP servers: [survey](https://pinggy.io/blog/best_computer_use_mcp_servers/), [kwin-mcp](https://discuss.kde.org/t/kwin-mcp-mcp-server-for-gui-automation-using-kwins-eis-interface-and-virtual-sessions/44678)
- Linux proposal: [freedesktop mailing list, August 2025](https://lists.freedesktop.org/archives/freedesktop/attachments/20250802/5ae44102/attachment.htm)
- ActivityWatch: [architecture](https://docs.activitywatch.net/en/latest/architecture.html); Zeitgeist: [Arch Wiki](https://wiki.archlinux.org/title/Zeitgeist); Newton: [GNOME accessibility blog](https://blogs.gnome.org/a11y/)
