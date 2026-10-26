---
type: PlanningItem
id: PI-META-MCP-001
name: "The MCP surface of the stereotype work (GH #376, #377, #379)"
status: done
itemType: feature
achieves:
  - REQ-TRS-META-060
  - REQ-TRS-META-061
  - REQ-TRS-META-062
  - REQ-TRS-META-063
  - REQ-TRS-META-064
evidence:
  - ref: TC-TRS-META-060
  - ref: TC-TRS-META-061
  - ref: TC-TRS-META-062
  - ref: TC-TRS-META-063
  - ref: TC-TRS-META-064
  - path: repo:crates/syscribe/tests/mcp_stereotypes.rs
tags:
  - metadata
  - mcp
---

`get_element` returns resolved `stereotypes`; `search` and `list_by_type` take `metadata` clauses (and `list_by_type` `where`); new read tools `meta` and `gates`; the write tools refuse the built-in `StdMetadata` library by name. `list --json` and `get_element` share `tagged_values::applications_json`; `meta` and `meta --json` share `metacmd::report`.
