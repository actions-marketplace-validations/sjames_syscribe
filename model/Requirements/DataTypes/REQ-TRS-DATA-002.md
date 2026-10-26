---
type: Requirement
id: REQ-TRS-DATA-002
name: "A default and a subtype stay within the constraints"
status: draft
reqDomain: software
reqClass: system
tags:
  - data-types
---

A `default:` and a specialising `AttributeDef` shall satisfy the constraints they inherit (GH #366).

## Behavior

- A `default:` outside the type's effective `range:`, or not matching its `pattern:`, is `E681` on the type.
- A subtype inherits its supertype's `range:` and `pattern:`. It may narrow the range; a range not contained in the supertype's is `E683`. A different `pattern:` on a subtype is `E683` (a pattern cannot be shown to narrow another).
- A supertype chain that is a cycle or does not resolve is left to the existing reference checks.
