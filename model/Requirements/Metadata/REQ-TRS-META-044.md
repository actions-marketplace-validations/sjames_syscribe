---
type: Requirement
id: REQ-TRS-META-044
name: "All renderers agree on stereotype style"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

The browser, static SVG and Mermaid/PlantUML exports shall read the same style (GH #378).

## Behavior

- One resolved style model feeds every renderer; a renderer that cannot express a property (e.g. an icon in Mermaid) degrades to the nearest it can and documents it.
