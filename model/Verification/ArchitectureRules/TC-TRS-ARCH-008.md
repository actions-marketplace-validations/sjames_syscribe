---
type: TestCase
id: TC-TRS-ARCH-008
name: "rule waivers"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules_2.rs
verifies:
  - REQ-TRS-ARCH-008
tags:
  - architecture-rules
---

```gherkin
Feature: rule waivers

  Scenario: current
    Then a waiver suppresses the finding

  Scenario: expired or unused
    Then the finding returns and W664 names the waiver

  Scenario: malformed
    Then a missing rationale is E662
```
