---
type: PlanningItem
id: PI-DEPL-370
name: "Deployment structure, slice one: node kinds and checks (GH #370)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-DEPL-001
  - REQ-TRS-DEPL-002
  - REQ-TRS-DEPL-003
evidence:
  - ref: TC-TRS-DEPL-001
  - ref: TC-TRS-DEPL-002
  - ref: TC-TRS-DEPL-003
  - path: repo:crates/syscribe-model/tests/deployment.rs
tags:
  - deployment
---

`deploymentKind` on parts, nesting order (`E731`), a component deployed once unless `replicated` (`E732`), host versus hosted integrity (`E733`), in `crates/syscribe-model/src/deployment.rs`. The STM32G474 of the grid-tie model is declared a `soc`. Still open in #370: the derived Deployment diagram (nested nodes, communication links), the OS-compatibility check, a component allocated to nothing, and per-node roll-up of the budgets.
