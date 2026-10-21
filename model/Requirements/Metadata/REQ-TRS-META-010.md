---
type: Requirement
id: REQ-TRS-META-010
name: "Tagged values are type-, unit-, range- and enumeration-checked"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A `MetadataDef` shall be able to constrain the value of each tagged value, and an application violating the declaration shall be reported (GH #375).

## Behavior

- A `MetadataDef` feature may declare `typedBy` (a scalar type, an enumeration, or an `ItemDef`), `unit`, `range` and `enumValues`. Features that declare none behave as today (name check `W045` only).
- A value of the wrong scalar type, or with a unit of a different dimension, is `E322`. A value outside `range` is `E323`. A value not in `enumValues` is `E324`.
- Each finding is reported on the applying element's file and names the stereotype and the key.
- `{type: Safety, integrity: "banana"}` against an integer `integrity` is `E322`.
- A bare reference application, and a definition that declares no typing, produce no new finding.
