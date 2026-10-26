---
type: Requirement
id: REQ-TRS-META-061
name: "MCP search and list_by_type filter by stereotype"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
  - mcp
---

The MCP `search` and `list_by_type` tools shall filter by stereotype as the CLI `list` does (GH #376).

## Behavior

- Both take `metadata: [clause, …]`: every clause must hold; a clause is a comma list of alternatives, any of which may hold; a leading `!` negates; a specialisation counts. A name that is no `MetadataDef` of the model and that no element applies is a tool error.
- `list_by_type` also takes `where`, with the `custom.`/`meta.` predicates `search` already takes.
- Filters apply before `limit`/`offset`, and `total` counts the filtered set.
