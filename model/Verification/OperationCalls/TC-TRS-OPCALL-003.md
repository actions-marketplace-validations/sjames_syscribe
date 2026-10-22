---
type: TestCase
id: TC-TRS-OPCALL-003
name: "A call's argument values have the parameter's kind and unit"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/operation_calls.rs
verifies:
  - REQ-TRS-OPCALL-003
tags:
  - operations
---

```gherkin
Feature: A call's argument values have the parameter's kind and unit

  Scenario: kind
    Then a value of the wrong kind is E675

  Scenario: unit
    Then a value of the wrong unit dimension is E675 and an equivalent unit is not
```
