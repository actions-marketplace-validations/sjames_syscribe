---
type: TestCase
id: TC-TRS-META-024
name: "A common selector grammar names elements by stereotype"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_selector.rs
verifies:
  - REQ-TRS-META-024
tags:
  - metadata
---

```gherkin
Feature: A common selector grammar names elements by stereotype

  Scenario: selector semantics
    Given {stereotype: Def, where: key>=2}
    Then it selects elements applying Def or a specialisation with the value satisfied

  Scenario: consumers
    Then diagram include, ArchitectureRule scopes, matrix filters and test plan selection accept it

  Scenario: unknown definition or key
    Then the consumer's unresolved-reference finding is reported
```
