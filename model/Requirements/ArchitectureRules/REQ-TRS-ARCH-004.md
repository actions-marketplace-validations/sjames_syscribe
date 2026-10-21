---
type: Requirement
id: REQ-TRS-ARCH-004
name: "forbid-dependency finds the path"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

A `forbid-dependency` rule shall report each forbidden dependency with the path that makes it (GH #369).

## Behavior

- A violation is a component chosen by `from` that depends, directly or through a chain, on a component chosen by `to`.
- A path ends at the first forbidden component and does not run through another `from` component (that one reports for itself), so one wiring mistake is one finding. The finding is on the rule's file, names the rule and the shortest path (`A → B → C`); `E660` for `severity: error`, `W660` for `warning`.
- On the grid-tie model, `AR-GTI-001` (trip-path hardware must not depend on software) passes, and fails with `PowerStage → Firmware` once the firmware is wired to `powerStage.hwDisable`.
- A chain of two connections is found; removing the last link clears the finding.
