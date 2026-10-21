---
type: Requirement
id: REQ-TRS-OPS-003
name: "A realising action's signature matches the operation"
status: draft
reqDomain: software
reqClass: system
tags:
  - operations
---

The signature of the realising `ActionDef` shall match the operation's, and each difference shall be reported (GH #364, item 2).

## Behavior

- Each operation parameter has an action parameter of the same name, with the same `direction` (default `in`), the same `typedBy` when both give one, and the same `unit` when both give one; an action parameter that is not an operation parameter and is not the return is extra.
- The operation's `returnType` corresponds to the action's one `direction: return` parameter: both or neither, and the same type when both give one. An asynchronous operation has no return.
- Each difference is one `E672` on the element that declares the operation, naming the operation, the action and the parameter.
