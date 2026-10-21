---
type: Requirement
id: REQ-TRS-META-042
name: "Edge stereotypes style connections"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

Stereotypes on connections shall colour and dash the edge (GH #378, #335).

## Behavior

- The same style mapping applies to an edge of a derived IBD whose `connections:` entry applies the stereotype (`metadata:`) or whose `typedBy` `ConnectionDef` does; `stroke`, `width` and `dash` apply (`fill` has no meaning for an edge).
