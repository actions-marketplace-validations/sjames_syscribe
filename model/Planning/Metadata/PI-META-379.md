---
type: PlanningItem
id: PI-META-379
name: "Stereotype-conditioned checks, gates and reports (GH #379)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-META-050
  - REQ-TRS-META-051
  - REQ-TRS-META-052
  - REQ-TRS-META-053
  - REQ-TRS-META-054
tags:
  - metadata
---

Done: `checks:` on a `MetadataDef` with `E650`/`W650` (050), `validate --deny-meta` and `[gates] deny_meta` (051), the `meta` report (052), and the baseline refusal (054). Still to do: selectors and per-stereotype coverage thresholds in architecture rules, views and `matrix`/`coverage` (053), and coverage-based gates.
