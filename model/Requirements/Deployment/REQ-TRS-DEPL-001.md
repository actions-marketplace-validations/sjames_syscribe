---
type: Requirement
id: REQ-TRS-DEPL-001
name: "Deployment nodes have a kind and nest in order"
status: draft
reqDomain: software
reqClass: system
tags:
  - deployment
---

A `PartDef`/`Part` shall be able to declare what kind of deployment node it is, and nodes shall nest in a valid order (GH #370, item 1).

## Behavior

- `deploymentKind:` is one of `soc`, `core`, `partition`, `container`, `process`, `thread` (outermost first). Any other value is `E730`.
- A node that contains (through `features:`, by `typedBy`) another node must contain only a kind further in: a `process` in a `partition` is fine, a `partition` in a `process`, or a node in a node of the same kind, is `E731`, naming both.
- `deploymentKind` on any other element type is `W047`.
