---
type: Requirement
id: REQ-TRS-BUD-003
name: "A budgets report shows use against capacity"
status: draft
reqDomain: software
reqClass: system
tags:
  - budgets
---

`syscribe budgets` shall report use against capacity per hardware part and resource (GH #368, item 3).

## Behavior

- `syscribe -m <root> budgets [--json]` lists, for every hardware part that declares a capacity, each resource with used, capacity, margin, limit, utilisation and the contributors, largest first. `--json` is `{budgets: [{element, resource, usedBytes, capacityBytes, marginPercent, limitBytes, utilisation, contributors: [{element, bytes}]}]}`.
- The report and `validate` share one computation. With nothing declared the report says so and exits 0.
