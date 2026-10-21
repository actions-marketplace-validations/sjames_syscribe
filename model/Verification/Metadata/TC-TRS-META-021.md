---
type: TestCase
id: TC-TRS-META-021
name: "--metadata accepts several definitions and negation"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_query.rs
verifies:
  - REQ-TRS-META-021
tags:
  - metadata
---

```gherkin
Feature: --metadata accepts several definitions and negation

  Scenario: both
    When --metadata A --metadata B
    Then only elements applying both are listed

  Scenario: either
    When --metadata A,B
    Then elements applying either are listed

  Scenario: negation
    When --metadata '!A'
    Then elements not applying A are listed

  Scenario: specialisation
    Then an element applying a sub-definition matches its parent

  Scenario: unknown definition
    Then it is reported rather than silently empty
```
