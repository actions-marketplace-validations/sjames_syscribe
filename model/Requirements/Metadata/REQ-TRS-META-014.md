---
type: Requirement
id: REQ-TRS-META-014
name: "A MetadataDef may constrain relations between its tagged values"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A `MetadataDef` shall carry `constraints:` evaluated per application with the expression language (GH #375, #273).

## Behavior

- `constraints:` is a list of boolean expressions over the definition's own tagged values, e.g. `integrity >= 2 implies goal != ""`.
- A constraint evaluating false is `E327`, naming the definition, the expression and the applying element. An expression that does not parse or type-check is reported on the definition.
- Defaults (REQ-TRS-META-012) are in scope. An absent String reads as `""` and an absent Boolean as `false`, so `goal != ""` is decidable; a constraint that needs an absent number is undecided, not a violation.
- `A implies B` is accepted and means `not (A) or (B)`.
- The constraint is not evaluated on a bare reference application of a definition that has no required values.
