---
type: Requirement
id: REQ-TRS-OPS-002
name: "An operation is realised by an ActionDef"
status: draft
reqDomain: software
reqClass: system
tags:
  - operations
---

An operation shall be able to name the `ActionDef` that realises it (GH #364, item 2).

## Behavior

- `realizedBy: <ActionDef>` on an `operations:` entry names the behaviour that implements it.
- A reference that does not resolve, or resolves to anything other than an `ActionDef`, is `E671`.
- An operation without `realizedBy` is unaffected.
