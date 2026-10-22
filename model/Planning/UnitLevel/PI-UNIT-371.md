---
type: PlanningItem
id: PI-UNIT-371
name: "Unit level and checks, slice one (GH #371)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-UNIT-001
  - REQ-TRS-UNIT-002
evidence:
  - ref: TC-TRS-UNIT-001
  - ref: TC-TRS-UNIT-002
  - path: repo:crates/syscribe-model/tests/unit_level.rs
tags:
  - unit-level
---

`level: component | unit` with the unit checks. The `design-report` follows in a second slice.
