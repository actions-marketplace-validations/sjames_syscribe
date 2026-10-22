---
type: Requirement
id: REQ-TRS-BUD-006
name: "CPU utilisation of what is allocated to a processor is checked against a bound"
status: draft
reqDomain: software
reqClass: system
tags:
  - budgets
---

`validate` shall check the CPU utilisation of the tasks allocated to a hardware part (GH #368, item 2).

## Behavior

- A hardware part opts in with `cpuMargin` (percent, 0–100) and/or `scheduling` (`edf`, the default, or `rate-monotonic`).
- Its tasks are the components allocated to it and everything they contain (each once) that declare a `period` and a `wcet` duration. Utilisation is the sum of `wcet / period`.
- The bound is 1 for `edf` and the Liu–Layland `n (2^(1/n) − 1)` for `rate-monotonic` over `n` tasks; the limit is the bound × `cpuMargin` / 100.
- Utilisation above the limit is `E692`, naming the part, the utilisation, the limit and the tasks, largest first. Equal passes.
- `syscribe budgets` reports it under `cpu` (JSON) with the tasks.
