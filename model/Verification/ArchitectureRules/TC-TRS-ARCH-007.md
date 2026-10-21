---
type: TestCase
id: TC-TRS-ARCH-007
name: "max-depth rules"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules_2.rs
verifies:
  - REQ-TRS-ARCH-007
tags:
  - architecture-rules
---

```gherkin
Feature: max-depth rules

  Scenario: depth
    Then a chain longer than maxDepth is E663 with the chain

  Scenario: cycles
    Then a cycle terminates
```
