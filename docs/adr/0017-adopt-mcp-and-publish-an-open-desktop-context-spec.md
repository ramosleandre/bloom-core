# 0017 · Adopt MCP on the consumer side; publish a desktop-context profile, not a new protocol

- **Date**: 2026-11-16
- **Status**: proposed
- **Deciders**: Everyone

## Context and problem

The context engine ([ADR 0012](0012-context-engine-as-a-standalone-daemon.md)) needs a data model
and protocols. Either an existing standard covers the need, or we define our own; and if we define
our own, we must decide how much of it to make public.

A survey ([`sense/spec/08-open-protocol.md`](../../sense/spec/08-open-protocol.md)) found:

- The Model Context Protocol is the settled way to deliver context to AI applications, under
  neutral governance.
- Platform owners are building mechanisms by which applications describe themselves to agents, and
  most shape them after MCP (Windows on-device registry, Android AppFunctions, WebMCP in Chrome;
  Apple keeps App Intents). All are in preview. None covers the Linux desktop.
- Many open-source MCP servers already expose one platform's accessibility tree. Each has its own
  output shape; none has a shared schema, a consent model or an opt-out.
- Nothing standardizes the shape of "what the user is doing now", nor the conduct expected of
  software that reads other applications.

## Options considered

1. A private, ad hoc protocol.
2. A new public protocol covering providers, consumers and schema ("Desktop Context Protocol").
3. MCP on the consumer side; our source protocol kept internal; and a small public **profile on
   MCP** for the uncovered part: the schema of desktop context, naming conventions for resources
   and tools, and rules of conduct (on-request by default, visible indicator, opt-outs honoured).

## Chosen option

3.

- An MCP server (`sense-mcp`) exposes the engine to any MCP-capable assistant. Off by default.
- The native local API stays for first-party clients (latency budget, event stream, scopes); it
  shares the schema, so the MCP server is a mapping.
- The source protocol is an internal interface for our own extensions. Third-party applications
  are consumed through whatever they already speak, as additional sources, when those mechanisms
  mature.
- The profile is written after the schema has survived two platforms and the browser source, and
  is budgeted in days. Whether to publish it in its own repository is decided then.

## Rationale

- Option 2 would compete with protocols the industry and the platform owners have already chosen.
  No application author would implement a provider protocol from this project.
- The uncovered part is real and small: a schema and a code of conduct. Most of it must be written
  for our own agents anyway.
- An MCP server makes the engine useful to assistants other than Bloom without asking them to
  adopt anything new, which is the only realistic path to outside use.
- The Linux desktop has no platform owner to provide this; a well-behaved cross-desktop server is
  a credible open-source contribution there.

## Accepted consequences

- Two entry points to keep consistent (native API and MCP server); mitigated by one schema.
- An MCP client normally forwards what it reads to a hosted model: that door needs its own consent
  step, scopes and visibility.
- A published schema is a compatibility commitment; publication waits until it is stable.
- Outside adoption is unlikely and is not a goal the project depends on.
- Standards work must not delay the engine or Bloom's vertical slice; it is the last stage of the
  work packages.
