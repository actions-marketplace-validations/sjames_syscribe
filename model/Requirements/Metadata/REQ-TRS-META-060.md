---
type: Requirement
id: REQ-TRS-META-060
name: "MCP get_element returns the resolved stereotype applications"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
  - mcp
---

The MCP `get_element` tool shall return an element's stereotype applications as resolved data, as `list --json` does (GH #376).

## Behavior

- When the element has applications, the result (summary, detail and `fields` forms) carries `stereotypes: [{def, values, defaulted, derived?}]`: `def` the resolved definition's qualified name, `values` the written values plus declared defaults (inherited ones included), `defaulted` the keys that came from a default, `derived: true` for an application a profile's `autoApply:` rule gives the element.
- With no applications the key is absent.
- The raw `metadata:` stays under `frontmatter`, as written.
