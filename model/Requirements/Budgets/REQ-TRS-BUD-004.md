---
type: Requirement
id: REQ-TRS-BUD-004
name: "A software part declares its timing"
status: draft
reqDomain: software
reqClass: system
tags:
  - budgets
---

A software `PartDef`/`Part` shall be able to declare its timing, and the values shall be checked (GH #368, item 1).

## Behavior

- `period` and `deadline` are durations (`10 ms`, `120 us`, `0.5 s`); `wcet` on a part is read as a duration when it is one — an opaque claim such as `O(1)` is left alone; `priority` is an integer.
- A `period` or `deadline` that is not a positive duration, or a `priority` that is not an integer, is `E691`.
- The fields are recognised on a `PartDef` or `Part`; on any other element type they are `W047`.
