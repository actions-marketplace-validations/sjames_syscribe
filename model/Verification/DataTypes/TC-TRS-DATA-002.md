---
type: TestCase
id: TC-TRS-DATA-002
name: "A default and a subtype stay within the constraints"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/data_types.rs
verifies:
  - REQ-TRS-DATA-002
tags:
  - data-types
---

```gherkin
Feature: A default and a subtype stay within the constraints

  Scenario: default
    Then a default outside the range or not matching the pattern is E681

  Scenario: subtype
    Then narrowing is accepted and widening or another pattern is E683
```
