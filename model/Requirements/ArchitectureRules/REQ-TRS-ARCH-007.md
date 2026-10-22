---
type: Requirement
id: REQ-TRS-ARCH-007
name: "max-depth rules"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

An `ArchitectureRule` shall be able to bound the length of a dependency chain (GH #369).

## Behavior

- `kind: max-depth` with `maxDepth: N` (a non-negative integer) and an optional `from` (default: every component): the longest chain of dependency edges starting at a chosen component, over the rule's `via`, must be at most N. Longer is `E663` (`W663` by severity), naming the rule, the start, the depth and the chain. A dependency cycle neither hangs nor is counted as unbounded: each component is visited once on a chain.
- A missing or non-integer `maxDepth` is `E662`.
