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

- `show --json`, `export` and the MCP read tools list applications as `{def, values}`, with defaulted values marked and the resolved definition's qualified name.
- Output for an element with no applications is unchanged (no empty key is added).
