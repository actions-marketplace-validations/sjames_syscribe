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

- `frame = true` on a `[diagram.stereotypes.<Def>]` makes it a frame criterion; an IBD with `groupBy: stereotype` gathers the parts that apply it (the part's, or its definition's) into one swimlane labelled with the stereotype's name, strongest first. A part with none stays in the boundary.
- Grouping honours specialisation.
