---
type: Requirement
id: REQ-TRS-OPCALL-003
name: "A call's argument values have the parameter's kind and unit"
status: draft
reqDomain: software
reqClass: system
tags:
  - operations
---

The value of each argument of a call that names an operation shall be of the kind and unit dimension its parameter declares (GH #364 item 3, #398).

## Behavior

- An argument's `value` is an expression of the model's expression language, typed statically: a number with an optional unit (`2 s`), a boolean, or a string. Names it uses are not typed.
- The parameter's kind and unit come from its `typedBy` and `unit`: a number type, `Boolean` or `String`; a number type with a `unit` has that dimension, one without is dimensionless.
- A value of another kind (a number for a boolean, a string for a number) or of a different unit dimension (`2 m` for a duration) is `E675`, naming the call, the parameter and what was given.
- A parameter whose type is not one of these, or a value that cannot be typed, is not checked. As with every unit check, dimensions are compared, not exact units (`2 ms` fills a parameter in `s`).
