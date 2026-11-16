# Architecture decision records

One file per structural decision, numbered in the order the number was taken. Template:
[`0000-template.md`](0000-template.md). An accepted ADR is never edited; a later one supersedes it.

| No. | Decision | Status | Where |
|---|---|---|---|
| 0001 | A new repository | proposed | `main` |
| 0002 | A model-agnostic decision port | proposed | `main` |
| 0003 | A core and four blocks | proposed | `main` |
| 0004 | A Python daemon and a Tauri shell | proposed | `main` |
| 0005 | Quality tooling | proposed | `main` |
| 0006 | No workflow engine in V1 | proposed | `main` |
| 0007 | Bloom targets Linux and macOS | proposed | `main` |
| 0008 to 0011 | Reserved for the interface block's prototype (cursor overlay, desktop cursor, GNOME adapter, window controls) | on branch `feature/tauri-ui` | not on `main` yet |
| 0012 | Context acquisition is a standalone local engine, in Rust | proposed | this proposal |
| 0013 | Read on request by default; ambient awareness is opt-in; never record | proposed | this proposal |
| 0014 | One source contract; one schema for every language | proposed | this proposal |
| 0015 | On Linux, session families detected at run time (amends 0007) | proposed | this proposal |
| 0016 | Privacy model of the context engine | proposed | this proposal |
| 0017 | MCP on the consumer side; a small public profile, not a new protocol | proposed | this proposal |
| 0018 | Acting through a closed list of effects | proposed | this proposal |

## Numbering

- Take the next free number **across all open branches**, not only `main`: check this table and
  `git branch -r` first.
- 0008 to 0011 are left free on purpose. The branch `feature/tauri-ui` uses them. That branch
  also contains an ADR numbered 0007 ("An interactive shell over the existing Interface port"),
  which collides with the 0007 already on `main`; it needs a new number when that branch is
  rebased (0019 is the next free one).
- Update this table in the same pull request that adds an ADR.
