---
type: Requirement
id: REQ-TRS-DATA-003
name: "A bound value is checked against its type's constraints"
status: draft
reqDomain: software
reqClass: system
tags:
  - data-types
---

A value bound to a feature typed by a constrained `AttributeDef` shall satisfy it (GH #366, item 2).

## Behavior

- For an inline `features:` entry or an `Attribute` element whose `typedBy` resolves to an `AttributeDef` with an effective `range:` or `pattern:`, a `value:` outside the range or not matching the pattern is `E682`, naming the feature, the value and the constraint.
- A numeric value may be written as a number or as text, with a unit; a unit is converted to the type's `unit:` when both are known and of one dimension.
- A feature with no `value:`, or typed by an unconstrained type, is not checked.
- A model that declares none of `range:`, `pattern:` and `default:` sees no new finding.
