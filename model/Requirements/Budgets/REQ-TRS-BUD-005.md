---
type: Requirement
id: REQ-TRS-BUD-005
name: "A task's worst-case time must fit its deadline"
status: draft
reqDomain: software
reqClass: system
tags:
  - budgets
---

A part whose `wcet` exceeds its `deadline` shall be reported (GH #368, item 2).

## Behavior

- A part with a `wcet` duration and a `deadline` where `wcet > deadline` is `E693`, naming both. Equal passes. A part that declares only one of them is not checked.
