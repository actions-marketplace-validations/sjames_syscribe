---
type: TestCase
id: TC-TRS-BUD-001
name: "Software parts declare their static memory and hardware its capacity"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/budgets.rs
verifies:
  - REQ-TRS-BUD-001
tags:
  - budgets
---

```gherkin
Feature: Software parts declare their static memory and hardware its capacity

  Scenario: sizes
    Then bytes and unit-bearing sizes are accepted

  Scenario: bad sizes
    Then an ambiguous or unknown unit, a negative or non-size value and a margin outside 0-100 are E691

  Scenario: other element types
    Then the fields are W047
```
