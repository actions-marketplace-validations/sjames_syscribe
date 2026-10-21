---
type: Requirement
id: REQ-TRS-META-040
name: "Diagram style is driven by stereotypes"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A `DiagramStyle` element or `[diagram.styles]` shall map a stereotype to presentation (GH #378).

## Behavior

- A style names a definition and optionally a tagged-value condition, and sets fill, stroke, dash, line width, icon, badge text and banner on/off.
- Several stereotypes on one element compose by a documented precedence: explicit order of declaration in the style set, later wins per property.
- With no style configured, output is byte-for-byte what it is today.
