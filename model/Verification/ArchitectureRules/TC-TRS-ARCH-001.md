---
type: TestCase
id: TC-TRS-ARCH-001
name: "An ArchitectureRule is a model element"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules.rs
verifies:
  - REQ-TRS-ARCH-001
tags:
  - architecture-rules
---

```gherkin
Feature: An ArchitectureRule is a model element

  Scenario: a well-formed rule
    Then it validates with no finding

  Scenario: malformed rules
    Then an unknown kind, a missing from or to, a bad via or severity and a bad selector are E662

  Scenario: a draft rule
    Then it is not evaluated

  Scenario: other element types
    Then kind, from, to and via are W047
```
