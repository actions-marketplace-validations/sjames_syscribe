---
type: TestCase
id: TC-TRS-OPS-001
name: "An operation declares the errors it can raise"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/operations_binding.rs
verifies:
  - REQ-TRS-OPS-001
tags:
  - operations
---

```gherkin
Feature: An operation declares the errors it can raise

  Scenario: resolving value types
    Then errors naming an ItemDef or an enumeration are quiet

  Scenario: bad errors
    Then an unresolved or non-value-type error is E670, and errors on an async operation

  Scenario: both returnType and errors
    Then a sync operation may declare both
```
