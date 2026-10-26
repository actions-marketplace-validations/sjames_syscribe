---
type: Requirement
id: REQ-TRS-DEPL-003
name: "A node must be at least as trustworthy as what it hosts"
status: draft
reqDomain: software
reqClass: system
tags:
  - deployment
---

A deployment node shall not host a component of higher integrity than the node declares (GH #370, item 3).

## Behavior

- When a node declares `asilLevel` (or `silLevel`) and a component allocated to it declares a higher level of the same kind, that is `E733`, naming both. Order: `QM` < `A` < `B` < `C` < `D`; SIL by number.
- A node that declares no level, or a component that declares none, is not checked.
