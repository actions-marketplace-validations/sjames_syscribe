---
type: TestCase
id: TC-TRS-META-023
name: "Expressions read stereotypes with meta() and has()"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_expr.rs
verifies:
  - REQ-TRS-META-023
tags:
  - metadata
---

```gherkin
Feature: Expressions read stereotypes with meta() and has()

  Scenario: has
    Then has(Safety) is true for an ASIL_D application of a sub-definition and false otherwise

  Scenario: meta value
    Then meta(SafetyCritical).integrity yields the value or its default

  Scenario: absent stereotype
    Given an element without the stereotype
    Then meta(Def).key is an absent value, not an error
    And a comparison against it is false

  Scenario: contexts
    Then it evaluates in constraints, view guards, scenario checks and eval
```
