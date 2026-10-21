---
type: TestCase
id: TC-TRS-ARCH-003
name: "Dependencies are derived from the architecture"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules.rs
verifies:
  - REQ-TRS-ARCH-003
tags:
  - architecture-rules
---

```gherkin
Feature: Dependencies are derived from the architecture

  Scenario: connections
    Then a connection makes the consumer depend on the producer

  Scenario: typedBy
    Then a composite depends on the definitions of its usages

  Scenario: allocations
    Then an allocated source depends on its target

  Scenario: via
    Then only the listed edge sources contribute
```
