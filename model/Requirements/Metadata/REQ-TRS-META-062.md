---
type: Requirement
id: REQ-TRS-META-062
name: "MCP meta tool reports stereotype applications"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
  - mcp
---

An MCP `meta` tool shall return the `syscribe meta` report (GH #379).

## Behavior

- `meta {def?, group_by?}` returns the same JSON as `syscribe meta --json`: the applications per definition, the elements of one definition, or those grouped by a tagged value.
- An unknown `def` is a tool error. The tool is read-only.
