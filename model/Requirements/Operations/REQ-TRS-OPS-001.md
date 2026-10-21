---
type: Requirement
id: REQ-TRS-OPS-001
name: "An operation declares the errors it can raise"
status: draft
reqDomain: software
reqClass: system
tags:
  - operations
---

An operation shall be able to declare the error types it can raise, and each shall be checked (GH #364, item 1).

## Behavior

- `errors:` on an `operations:` entry (of any element type) is a reference or a list of references to value types: an `ItemDef`, `AttributeDef`, `EnumerationDef` or `Enumeration`.
- A reference that does not resolve, or resolves to anything else, is `E670`.
- A synchronous operation may declare both `returnType` and `errors`. An asynchronous one (`isAsync: true`) returns nothing, so declaring `errors` on it is `E670`.
- An operation that declares no `errors` is unaffected.
