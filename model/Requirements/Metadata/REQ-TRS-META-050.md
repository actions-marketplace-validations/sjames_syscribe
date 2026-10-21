---
type: Requirement
id: REQ-TRS-META-050
name: "A MetadataDef declares checks run on every carrier"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A `MetadataDef` shall declare `checks:` evaluated on each element carrying it (GH #379).

## Behavior

- A check is an expression plus a message and severity; it is evaluated against the carrying element with `meta()` in scope. Applications of sub-definitions run the inherited checks.
- A failing check is reported as `E650` (error) or `W650` (warning) naming the definition and element; passing elements are quiet. Warnings are draft-suppressed like other warnings.
- Checks run only for enabled profiles (REQ-TRS-META-031) or definitions in the model itself.
