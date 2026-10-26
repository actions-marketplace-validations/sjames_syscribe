---
type: TestCase
id: TC-TRS-ARCH-004
name: "forbid-dependency finds the path"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules.rs
verifies:
  - REQ-TRS-ARCH-004
tags:
  - architecture-rules
---

```gherkin
Feature: forbid-dependency finds the path

  Scenario: a forbidden dependency through two connections
    Then it is reported with the whole path

  Scenario: removing the link
    Then the finding is gone

  Scenario: severity
    Then error is E660 and warning is W660
```
