---
type: TestCase
id: TC-TRS-OPCALL-001
name: "A call names the operation it calls"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/operation_calls.rs
verifies:
  - REQ-TRS-OPCALL-001
tags:
  - operations
---

```gherkin
Feature: A call names the operation it calls

  Scenario: checks
    Then the findings of the requirement are raised
```
