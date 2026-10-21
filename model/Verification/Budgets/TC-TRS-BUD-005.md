---
type: TestCase
id: TC-TRS-BUD-005
name: "A task's worst-case time must fit its deadline"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/budgets_cpu.rs
verifies:
  - REQ-TRS-BUD-005
tags:
  - budgets
---

```gherkin
Feature: A task's worst-case time must fit its deadline

  Scenario: wcet above deadline
    Then E693 names both

  Scenario: equal or only one declared
    Then no finding
```
