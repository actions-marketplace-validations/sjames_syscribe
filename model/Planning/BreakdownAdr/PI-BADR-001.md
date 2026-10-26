---
type: PlanningItem
id: PI-BADR-001
name: "Make the breakdown ADR requirement optional"
status: done
itemType: feature
achieves:
  - REQ-TRS-BADR-001
evidence:
  - ref: TC-TRS-BADR-001
  - path: repo:crates/syscribe-model/tests/breakdown_adr.rs
tags:
  - traceability
---

`[traceability] require_breakdown_adr = false` switches `E310` off.
