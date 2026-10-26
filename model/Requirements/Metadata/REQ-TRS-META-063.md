---
type: Requirement
id: REQ-TRS-META-063
name: "MCP gates tool evaluates the project's stereotype gates"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
  - mcp
---

An MCP `gates` tool shall evaluate `[gates] deny_meta` and `[[gates.coverage]]` (GH #379).

## Behavior

- `gates {}` returns `{tripped, denyMeta: [{condition, element}], coverage: [line, …]}`, the same conditions `validate` gates on; `tripped` is true when either list is non-empty.
- A malformed gate is a tool error naming it. The tool is read-only; it never changes the model.
