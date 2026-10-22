---
type: TestCase
id: TC-TRS-OPCALL-002
name: "A call matches the operation's signature"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/operation_calls.rs
verifies:
  - REQ-TRS-OPCALL-002
tags:
  - operations
---

```gherkin
Feature: A call matches the operation's signature

  Scenario: checks
    Then the findings of the requirement are raised
```
