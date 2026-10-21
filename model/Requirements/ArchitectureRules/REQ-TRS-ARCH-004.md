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
- One finding per offending pair, on the rule's file, naming the rule and the shortest path (`A → B → C`); `E660` for `severity: error`, `W660` for `warning`.
- A chain of two connections is found; removing the last link clears the finding.
