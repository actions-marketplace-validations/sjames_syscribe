---
type: PlanningItem
id: PI-OPS-364
name: "Operations: errors and behaviour binding (GH #364, items 1 and 2)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-OPS-001
  - REQ-TRS-OPS-002
  - REQ-TRS-OPS-003
evidence:
  - ref: TC-TRS-OPS-001
  - ref: TC-TRS-OPS-002
  - ref: TC-TRS-OPS-003
  - path: repo:crates/syscribe-model/tests/operations_binding.rs
tags:
  - operations
---

Done: `errors:` on an operation (`E670`), `realizedBy:` (`E671`) and the signature match against the `ActionDef` (`E672`), in `crates/syscribe-model/src/operations.rs`. Still to do in #364: call-site checks (item 3), code-symbol links with drift detection (item 4), and a part's `performs:` naming operations of the interfaces it provides.
