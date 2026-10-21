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

- `checks:` on a `MetadataDef` is a list of entries, each a string expression or `{expr, message, severity}` with `severity` `error` (the default) or `warning`; it is evaluated against the carrying element with `has()`/`meta()` in scope. Applications of sub-definitions run the inherited checks.
- A failing check is reported as `E650` (error) or `W650` (warning) naming the definition and element; passing elements are quiet. Warnings are draft-suppressed like other warnings.
- Checks run only for enabled profiles (REQ-TRS-META-031) or definitions in the model itself.
- A malformed entry (no `expr:`, a bad `severity:`, an expression that does not parse, a call naming no stereotype) is `E650` on the definition.
