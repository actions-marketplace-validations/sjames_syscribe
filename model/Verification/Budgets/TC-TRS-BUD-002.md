---
type: TestCase
id: TC-TRS-BUD-002
name: "Memory used by what is allocated to an element is checked against its capacity"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/budgets.rs
verifies:
  - REQ-TRS-BUD-002
tags:
  - budgets
---

```gherkin
Feature: Memory used by what is allocated to an element is checked against its capacity

  Scenario: under and over
    Then 79 against an 80 limit passes and 81 is E690 naming the contributors

  Scenario: roll-up
    Then a composite counts its contained components once

  Scenario: stack
    Then stack counts toward RAM

  Scenario: allocation forms
    Then allocatedTo, allocatedFrom and an Allocation element all count

  Scenario: no capacity
    Then nothing is checked
```
