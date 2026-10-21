---
type: TestCase
id: TC-TRS-OPS-002
name: "An operation is realised by an ActionDef"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/operations_binding.rs
verifies:
  - REQ-TRS-OPS-002
tags:
  - operations
---

```gherkin
Feature: An operation is realised by an ActionDef

  Scenario: realizedBy resolves
    Then an ActionDef is accepted

  Scenario: bad target
    Then an unresolved reference or a non-ActionDef is E671
```
