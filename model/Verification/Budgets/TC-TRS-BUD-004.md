---
type: TestCase
id: TC-TRS-BUD-004
name: "A software part declares its timing"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/budgets_cpu.rs
verifies:
  - REQ-TRS-BUD-004
tags:
  - budgets
---

```gherkin
Feature: A software part declares its timing

  Scenario: durations and priority
    Then valid values are quiet

  Scenario: malformed
    Then a non-duration, a non-positive duration and a non-integer priority are E691

  Scenario: opaque wcet
    Then a wcet that is not a duration is ignored

  Scenario: other element types
    Then the fields are W047
```
