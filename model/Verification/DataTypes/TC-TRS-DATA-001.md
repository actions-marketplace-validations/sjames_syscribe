---
type: TestCase
id: TC-TRS-DATA-001
name: "An AttributeDef constrains its values"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/data_types.rs
verifies:
  - REQ-TRS-DATA-001
tags:
  - data-types
---

```gherkin
Feature: An AttributeDef constrains its values

  Scenario: well-formed constraints
    Then range, pattern and default validate

  Scenario: malformed
    Then a bad range, lo above hi or a bad pattern is E680

  Scenario: other element types
    Then the fields are W047
```
