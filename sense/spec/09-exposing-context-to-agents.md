# 09 · Exposing context to an AI agent

The other files say how Sense *obtains* context. This one says how context should be *handed to a
model*, which is a different problem with its own failure modes: too many tokens, stale or
ambiguous facts presented as certain, and third-party text that the model mistakes for
instructions. It also places Sense among the ways the industry exposes applications to agents.

Sources are at the end. Research figures are as reported by their authors; treat them as
directional.

## Three ways the industry exposes a computer to an agent

| Model | How | Examples | Strength | Weakness |
|---|---|---|---|---|
| **Scrape** | The agent side reads what applications show: accessibility tree, DOM, or pixels. | Computer-use agents; dozens of open-source MCP servers (Peekaboo, Windows-MCP, linux-desktop-mcp, kwin-mcp, Cua) | Works with every application today, no cooperation needed | Bulky, slow, brittle; reads more than needed; the application has no say |
| **Donate** | The application describes itself to agents: its current content and the functions it offers. | Apple App Intents; Android AppFunctions; Windows on-device registry of MCP connectors; WebMCP in Chrome | Precise, cheap, consented by the application's author | Requires every application to adopt it; each vendor has its own mechanism; mostly in preview |
| **Connect** | The agent talks to the service's API behind the application. | MCP connectors for mail, calendar, code hosts | Complete data, independent of what is on screen | Knows nothing about what the user is looking at right now |

Where this is going: every platform owner is building the **donate** model, and three of the four
(Windows, Android, the web) are shaping it after MCP. Apple keeps its own. None of them covers the
Linux desktop, and none of them answers the cross-application question "what is the user doing
right now", because donation is per application.

Where Sense sits: it is the piece that knows the **present situation** (which application, which
window, what is selected, what was just copied) and that can fetch depth through whichever model is
available for the application in focus. Today that is scrape plus our own extensions. As
applications start donating, Sense should consume those channels as additional sources
(`08-open-protocol.md`), not compete with them.

## What a model should receive

Findings that shape the rules below:

- Structured text beats pixels on cost; pixels are needed only where an interface has no structure
  (canvases, custom-drawn editors). The systems that currently score best on desktop benchmarks are
  **hybrids** that use structure first and vision as a fallback.
- A raw accessibility tree is too large to hand over. Compressing it to roughly a fifth of its
  size, by dropping structural noise, is reported to preserve task performance.
- Instructions hidden in on-screen content do mislead agents, including through invisible
  characters and off-screen elements. Prompt-level defenses (system prompt warnings, delimiters)
  help little. What works is architecture: the component that decides what to do must not take its
  instructions from observed content.

### Rule 1 · Small first, depth on request

`capture` returns a **digest**: the situation in a few hundred tokens. Depth is pulled with
`content.read`, by the client or by the model through a tool. Nothing large is ever pushed.

Two renderings of the same snapshot, both produced by `sense-core` so every client gets the same:

- `json`: the schema of `01-data-model.md`, for programs.
- `card`: a compact, deterministic text form for models. Target: under 300 tokens without content.

```
[desktop context · 2026-11-16 10:12 · linux/gnome]
app: Firefox (browser)
window: "Re: Thursday – Mail"
document: https://mail.example.org/…
focus: text field, editable
selection (exact, 34 chars): see <selection>
clipboard: text, 212 chars, copied 3 min ago from Files (not included)
recent: Files → Firefox (2 min) → Terminal (40 s) → Firefox
unavailable: document text (not requested)

<selection source="ext.browser" trust="third-party">
Can you confirm Thursday at 2 pm?
</selection>
```

### Rule 2 · Say how sure each fact is

The card states quality in words the model can use: `exact`, `ambient (may be stale or from
another window)`, `inferred`. What is unknown is listed as unavailable with its reason. A model
that is told "selection: unavailable (application not accessible)" behaves better than one given
an empty string.

### Rule 3 · Compress structure before it leaves the engine

`content.read` of kind `Visible` or `Structure` never returns a raw tree. The core applies one
shared reduction, whatever the platform:

- drop nodes that are invisible, off-screen, zero-sized or purely decorative;
- collapse containers that carry no role or label;
- keep role, label, value and state for controls; keep text; keep heading and list structure;
- give each kept node a short stable reference (`e17`) usable in a follow-up read;
- enforce a token budget, truncating by distance from the focused element, and say what was cut.

### Rule 4 · Sanitize text at the boundary

Applied by the core to every text value, before rendering in any form:

- remove invisible and control characters used for hidden instructions: the Unicode tag block,
  zero-width characters, bidirectional overrides;
- normalize whitespace; cap length; mark truncation;
- never include text from nodes the user cannot see.

This removes the cheap hidden-channel attacks. It does not make content trustworthy.

### Rule 5 · Content is data, structurally

- Every content value is typed, carries `sensitivity: content`, and in the card sits inside a
  tagged block with its source. Field values are never concatenated into prose.
- Sense's own descriptions (MCP tool descriptions, resource names) contain no observed text.
- This is hygiene, not a defense. The defense belongs to the client's architecture, and Sense's
  documentation for client authors must say so plainly:
  - decide among a **closed set of actions**, so that injected text can at worst change a ranking;
  - keep the component that chooses actions separate from the one that reads third-party content;
  - require the user's approval, on the real content, for anything that leaves the machine.
  Bloom already works this way (fixed catalog, criticality floor, approval gate).

### Rule 6 · Handles, not coordinates

Every entity a model may want to come back to (the window, the focused element, a node in a
structure read) has a stable id for the life of that entity. Follow-up reads, and an actuator
outside Sense, use ids. Screen coordinates are not exposed in the card.

### Rule 7 · Events are for programs, not for prompts

The event stream exists so a client can keep its own state. It is not fed to a model. What a model
sees of the past is the short `recent` line in the digest, at observation level 2 only.

## Consequences for the rest of the specification

- `sense-core` gains a rendering module (`card`) and the structure reduction; both are pure
  functions with fixture tests.
- `capture` and `content.read` accept `format: "json" | "card"`.
- The MCP server returns the card as text content and the JSON as structured content.
- Text sanitization is a new step of the privacy filter (`04-privacy-security.md`).

## Sources

- Hybrid perception on desktop benchmarks: [Microsoft Research, Hybrid Computer Use](https://microsoft.github.io/Hybrid-Computer-Use/); latency cost of accessibility trees: [OSWorld-Human](https://arxiv.org/pdf/2506.16042); perception trade-offs: [Notte glossary](https://www.notte.cc/glossary/ai-browser-agents/how-does-a-browser-agent-perceive-a-page-vision-vs-dom)
- Injection through on-screen content: [VPI-Bench](https://arxiv.org/abs/2506.02456), [architectural isolation for computer-use agents](https://arxiv.org/abs/2601.09923), [Cloud Security Alliance research note](https://labs.cloudsecurityalliance.org/research/csa-research-note-computer-use-agent-safety-blindspots-20260/)
- Donation mechanisms: [Windows agentic platform](https://blogs.windows.com/windowsdeveloper/2025/11/18/ignite-2025-furthering-windows-as-the-premier-platform-for-developers-governed-by-security), [Android AppFunctions](https://developer.android.com/ai/appfunctions?hl=en), [WebMCP in Chrome](https://venturebeat.com/infrastructure/google-chrome-ships-webmcp-in-early-preview-turning-every-website-into-a), [WebMCP community notes](https://w3c-cg.github.io/aikr/webMCP/webmcp-technical-notes.html)
- Existing desktop MCP servers: [survey](https://pinggy.io/blog/best_computer_use_mcp_servers/), [kwin-mcp](https://discuss.kde.org/t/kwin-mcp-mcp-server-for-gui-automation-using-kwins-eis-interface-and-virtual-sessions/44678), [linux-desktop-mcp](https://glama.ai/mcp/servers/BeckhamLabsLLC/linux-desktop-mcp)
