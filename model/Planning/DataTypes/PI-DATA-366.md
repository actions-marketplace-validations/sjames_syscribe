---
type: PlanningItem
id: PI-DATA-366
name: "Data types with value constraints (GH #366, item 2)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-DATA-001
  - REQ-TRS-DATA-002
  - REQ-TRS-DATA-003
evidence:
  - ref: TC-TRS-DATA-001
  - ref: TC-TRS-DATA-002
  - ref: TC-TRS-DATA-003
  - path: repo:crates/syscribe-model/tests/data_types.rs
tags:
  - data-types
---

Done: `range:`, `pattern:` and `default:` on an `AttributeDef` (`E680`), a default and a subtype within them (`E681`, `E683`), and bound values checked against the effective constraints with unit conversion (`E682`), in `crates/syscribe-model/src/data_types.rs`. Still to do in #366: the derived Class diagram with attribute and enumeration compartments (items 1 and 3), `EnumerationDef` literals carrying values, and checking payload values in machine expressions and scenarios against the ranges.
