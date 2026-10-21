---
type: PlanningItem
id: PI-META-376
name: "Query and select by stereotype and tagged value (GH #376)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-META-020
  - REQ-TRS-META-021
  - REQ-TRS-META-022
  - REQ-TRS-META-023
  - REQ-TRS-META-024
  - REQ-TRS-META-025
tags:
  - metadata
---

Done: `--where meta.<Def>.<key>` on `list`/`find`/`ls`/`digest`/`stats`/MCP `search` (020), repeatable/comma/negated `--metadata` (021), `metadata` in `list --json` (022), `has()`/`meta()` in `MetadataDef` `constraints:` (023). Still to do: the selector grammar (024) and `appliesWhen:` on an application (025).
