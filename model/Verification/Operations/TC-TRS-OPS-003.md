---
type: TestCase
id: TC-TRS-OPS-003
name: "A realising action's signature matches the operation"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/operations_binding.rs
verifies:
  - REQ-TRS-OPS-003
tags:
  - operations
---

```gherkin
Feature: A realising action's signature matches the operation

  Scenario: matching
    Then no finding

  Scenario: each kind of mismatch
    Then a missing, extra, wrongly directed, mistyped or mis-united parameter and a return difference are each E672
```
