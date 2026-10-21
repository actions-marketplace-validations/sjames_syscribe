---
type: Requirement
id: REQ-TRS-BUD-001
name: "Software parts declare their static memory and hardware its capacity"
status: draft
reqDomain: software
reqClass: system
tags:
  - budgets
---

A `PartDef`/`Part` shall be able to declare the static memory it uses, and a hardware part the memory it offers (GH #368, item 1).

## Behavior

- Usage: `ram`, `flash`, `nvm` and `stack` (a size each). Capacity: `ramCapacity`, `flashCapacity`, `nvmCapacity` (a size each) and `margin` (the percentage of capacity that may be used, 0–100, default 100).
- A size is a number of bytes, or text with a unit: `B`, `kB`, `MB`, `GB` (powers of 1000) or `KiB`, `MiB`, `GiB` (powers of 1024), e.g. `48 KiB`, `1.5 MiB`. Any other unit (including `KB`, which is ambiguous), a negative number or text that is not a size is `E691`; so is a `margin` outside 0–100.
- The fields are recognised on a `PartDef` or `Part`; on any other element type they are `W047`.
- An element that declares none of them is not looked at.
