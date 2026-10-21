---
type: TestCase
id: TC-TRS-META-025
name: "A stereotype application may be variant-dependent"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_query.rs
verifies:
  - REQ-TRS-META-025
tags:
  - metadata
---

```gherkin
Feature: A stereotype application may be variant-dependent

  Scenario: guard false
    Given an application with appliesWhen on a feature
    When projected with a configuration not selecting it
    Then every consumer sees no application

  Scenario: no configuration
    Then all applications are present

  Scenario: diff
    Then diff --config A --config B reports the differing application
```
