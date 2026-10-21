---
type: Requirement
id: REQ-TRS-META-022
name: "Applications are exported as structured data"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

Stereotype applications shall be available as structured data in JSON and MCP output (GH #376).

## Behavior

- `list --json` lists applications under `metadata` as `{def, values, defaulted}`, with the resolved definition's qualified name. (`show` has no JSON mode; the MCP read tools return the same structure.)
- Output for an element with no applications is unchanged (no empty key is added).
