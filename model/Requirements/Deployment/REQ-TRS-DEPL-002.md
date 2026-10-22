---
type: Requirement
id: REQ-TRS-DEPL-002
name: "A component is deployed once unless replicated"
status: draft
reqDomain: software
reqClass: system
tags:
  - deployment
---

A software component allocated to more than one deployment node shall be reported unless it is marked replicated (GH #370, item 3).

## Behavior

- A `PartDef`/`Part` allocated (`allocatedTo:`, `allocatedFrom:` or an `Allocation`) to two or more distinct parts that declare a `deploymentKind` is `E732`, naming the component and the nodes. `replicated: true` on the component accepts it.
- Allocation to a part with no `deploymentKind` is not counted.
