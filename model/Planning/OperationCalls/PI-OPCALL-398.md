---
type: PlanningItem
id: PI-OPCALL-398
name: "Operation call sites, slice one (GH #398)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-OPCALL-001
  - REQ-TRS-OPCALL-002
evidence:
  - ref: TC-TRS-OPCALL-001
  - ref: TC-TRS-OPCALL-002
  - path: repo:crates/syscribe-model/tests/operation_calls.rs
tags:
  - operations
---

`operation:` on `SendAction`/`PerformAction`, resolved and checked against the operation's parameters. Value types and units, sequence-diagram messages and operation calls as architecture-rule dependencies follow.
