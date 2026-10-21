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

Done: `checks:` on a `MetadataDef` with `E650`/`W650` (050), `validate --deny-meta` and `[gates] deny_meta` (051), the `meta` report (052), and the baseline refusal (054). Selectors now work in a TestPlan, `matrix`, a diagram view's `include: {stereotypes}` and `[[coverage.rule]] stereotype` (053). Still to do: architecture rules (#369) and a coverage-based gate (e.g. every `Safety` requirement verified), which needs a design decision.
