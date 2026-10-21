---
type: TestCase
id: TC-TRS-META-014
name: "A MetadataDef may constrain relations between its tagged values"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_typed.rs
verifies:
  - REQ-TRS-META-014
tags:
  - metadata
---

```gherkin
Feature: A MetadataDef may constrain relations between its tagged values

  Scenario: violated constraint
    Given the constraint integrity >= 2 implies goal != ""
    When an application has integrity 3 and no goal
    Then E327 names the definition, the expression and the element

  Scenario: satisfied constraint is quiet
    When the application also has a goal
    Then nothing is reported

  Scenario: absent optional value
    When goal is absent and integrity is 1
    Then no finding is reported

  Scenario: unparsable expression
    Then the finding is reported on the definition, not the application
```
