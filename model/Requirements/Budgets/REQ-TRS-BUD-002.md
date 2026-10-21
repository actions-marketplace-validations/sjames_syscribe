---
type: Requirement
id: REQ-TRS-BUD-002
name: "Memory used by what is allocated to an element is checked against its capacity"
status: draft
reqDomain: software
reqClass: system
tags:
  - budgets
---

`validate` shall check the static memory of the software allocated to a hardware part against its capacity and margin (GH #368, item 2).

## Behavior

- For a hardware part that declares a capacity, the *contributors* are the components allocated to it (`allocatedTo:`, `allocatedFrom:` or an `Allocation` element) and, transitively, every component in their `features:` (the usages they contain), each counted once however many ways it is reached. The use is the sum of the contributors' own values; RAM use includes `stack`.
- Use above `capacity × margin / 100` is `E690`, naming the part, the resource, the use, the limit and the contributors, largest first. Use equal to the limit passes.
- A part with no capacity for a resource is not checked for it. The project default margin is `[budgets] margin` in `.syscribe.toml`; a part's own `margin` wins.
