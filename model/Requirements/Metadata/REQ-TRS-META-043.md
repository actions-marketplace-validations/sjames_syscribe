---
type: Requirement
id: REQ-TRS-META-043
name: "The diagram legend lists the styles used"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

The generator shall emit a legend from the styles actually drawn (GH #378).

## Behavior

- The legend lists exactly the stereotypes that styled something in the diagram, no more and no fewer, strongest first.
- It is omitted when no style applied.
