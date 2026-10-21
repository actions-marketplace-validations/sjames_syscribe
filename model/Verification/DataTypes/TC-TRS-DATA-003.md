---
type: TestCase
id: TC-TRS-DATA-003
name: "A bound value is checked against its type's constraints"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/data_types.rs
verifies:
  - REQ-TRS-DATA-003
tags:
  - data-types
---

```gherkin
Feature: A bound value is checked against its type's constraints

  Scenario: in and out of range
    Then an out-of-range value is E682 and an in-range one is quiet

  Scenario: pattern
    Then a non-matching string is E682

  Scenario: units
    Then a unit is converted to the type's unit

  Scenario: nothing declared
    Then no finding
```
