---
type: PlanningItem
id: PI-META-378
name: "Drive diagram style, grouping and legend from stereotypes (GH #378)"
status: done
itemType: feature
achieves:
  - REQ-TRS-META-040
  - REQ-TRS-META-041
  - REQ-TRS-META-042
  - REQ-TRS-META-043
  - REQ-TRS-META-044
evidence:
  - ref: TC-TRS-META-040
  - ref: TC-TRS-META-041
  - ref: TC-TRS-META-042
  - ref: TC-TRS-META-043
  - ref: TC-TRS-META-044
  - path: repo:crates/syscribe-model/tests/vis_stereotype.rs
  - path: repo:crates/syscribe-model/tests/vis_stereotype_frames.rs
tags:
  - metadata
---

`[diagram.stereotypes.<Def>]` (fill, stroke, dash, width, banner, frame, `when`, `priority`) applied to nodes and edges by `vis::stereo::apply` and folded into `style::node_style` / `edge_style_in`, so every writer agrees (040, 042, 044); the legend (043); `groupBy: stereotype` swimlanes (041). Not built: `icon` and `badge`.
