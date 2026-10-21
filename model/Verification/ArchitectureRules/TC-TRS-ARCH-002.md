---
type: TestCase
id: TC-TRS-ARCH-002
name: "Rule selectors choose components"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules.rs
verifies:
  - REQ-TRS-ARCH-002
tags:
  - architecture-rules
---

```gherkin
Feature: Rule selectors choose components

  Scenario: keys are ANDed
    Then package, tag, domain and stereotype select the components that satisfy all given

  Scenario: bad selector
    Then an unknown key or an empty selector is E662

  Scenario: vacuous
    Then a selector that chooses nothing is W662
```
