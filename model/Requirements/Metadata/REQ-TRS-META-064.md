---
type: Requirement
id: REQ-TRS-META-064
name: "MCP write tools refuse the built-in stereotype library"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
  - mcp
---

The MCP write tools shall refuse an element built into the tool, with a message that says so (GH #377).

## Behavior

- `update_element`, `delete_element`, `move_element` and the batch tool refuse `StdMetadata::…` (REQ-TRS-META-032) with a tool error stating that it is built into the tool and read-only, and to specialise it in a package of the project instead; nothing is written.
- The CLI `move` refuses it the same way.
