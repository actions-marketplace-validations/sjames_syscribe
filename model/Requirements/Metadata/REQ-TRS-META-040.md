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

- `[diagram.stereotypes.<Def>]` in `.syscribe.toml` names a definition (matching its specialisations too) and sets fill, stroke, dash, outline width and banner on/off, optionally under a tagged-value condition (`when`). Icon and badge text are not yet supported.
- Several stereotypes on one element compose per property by `priority` (higher wins), ties going to the later definition name.
- With no style configured, output is byte-for-byte what it is today.
