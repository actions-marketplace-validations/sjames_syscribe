---
type: PlanningItem
id: PI-BUD-368
name: "Resource budgets: memory (GH #368, items 1-3 for RAM/flash/NVM)"
status: in_progress
itemType: feature
achieves:
  - REQ-TRS-BUD-001
  - REQ-TRS-BUD-002
  - REQ-TRS-BUD-003
  - REQ-TRS-BUD-004
  - REQ-TRS-BUD-005
  - REQ-TRS-BUD-006
evidence:
  - ref: TC-TRS-BUD-001
  - ref: TC-TRS-BUD-002
  - ref: TC-TRS-BUD-003
  - ref: TC-TRS-BUD-004
  - ref: TC-TRS-BUD-005
  - ref: TC-TRS-BUD-006
  - path: repo:crates/syscribe-model/tests/budgets_cpu.rs
  - path: repo:crates/syscribe-model/tests/budgets.rs
  - path: repo:crates/syscribe/tests/budgets_cli.rs
  - path: repo:crates/syscribe/tests/budgets_gti.rs
tags:
  - budgets
---

Done: `ram`/`flash`/`nvm`/`stack` on software parts, `ramCapacity`/`flashCapacity`/`nvmCapacity`/`margin` on hardware, the roll-up check (`E690`, `E691`) and `syscribe budgets`, in `crates/syscribe-model/src/budgets.rs`. The grid-tie STM32G474 declares its 128 kB RAM, 512 kB flash and the 80 % margin of `REQ-GTI-SYS-072`; no firmware size is declared yet, so its use reads 0. Still to do in #368: CPU utilisation (`period`, `deadline`, `priority`, `wcet` with a schedulability bound, the 70 % CPU half of `REQ-GTI-SYS-072`), end-to-end latency along a chain, and linking a margin requirement to a budget.
