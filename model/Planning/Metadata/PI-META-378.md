---
type: PlanningItem
id: PI-META-378
name: "Drive diagram style, grouping and legend from stereotypes (GH #378)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-META-040
  - REQ-TRS-META-041
  - REQ-TRS-META-042
  - REQ-TRS-META-043
  - REQ-TRS-META-044
tags:
  - metadata
---

Done: `[diagram.stereotypes.<Def>]` (fill, stroke, dash, width, banner, `when`, `priority`) applied to every node by `vis::stereo::apply` and folded into `style::node_style`, so the browser, SVG, DOT and PlantUML agree and Mermaid gets a `style` line (040, 043, 044). Still to do: grouping a frame by stereotype (041), styling edges (042), icon and badge.
