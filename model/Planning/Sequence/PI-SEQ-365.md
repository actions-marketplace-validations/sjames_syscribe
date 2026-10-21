---
type: PlanningItem
id: PI-SEQ-365
name: "Sequence diagrams from parts and operations (GH #365)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-SEQ-001
tags:
  - diagram
  - sequence
---

Done: a `PartDef`/`Part` subject derives its sequence from the actions it `performs:` (item 1). Still to do in #365: call and return messages with operation-signature labels, async arrowheads, create/destroy (items 2, needs the call-site syntax of #364 item 3); `opt`, `alt` else dividers, `par`, `break` and loop guards (item 3, which also needs the writers and the sprotty client to draw dividers); timeouts (item 4).
