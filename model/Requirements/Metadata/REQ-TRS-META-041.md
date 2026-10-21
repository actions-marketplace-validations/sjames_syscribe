---
type: Requirement
id: REQ-TRS-META-041
name: "A stereotype can be a diagram grouping criterion"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A style shall be able to declare a stereotype as a frame criterion (GH #378).

## Behavior

- A frame style groups elements applying the stereotype (or sharing a tagged value) into a swimlane, as `domain` does.
- Grouping honours specialisation.
