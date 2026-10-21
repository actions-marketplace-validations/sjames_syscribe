---
type: TestCase
id: TC-TRS-ARCH-006
name: "allow-only and layering rules"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/arch_rules_2.rs
verifies:
  - REQ-TRS-ARCH-006
tags:
  - architecture-rules
---

```gherkin
Feature: allow-only and layering rules

  Scenario: allow-only
    Then a dependency outside from and to is E660

  Scenario: layering
    Then a dependency on a higher layer is E660 and skipping layers is allowed

  Scenario: malformed
    Then a missing layers list, or from/to on allow-only, is E662
```
