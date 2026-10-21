---
type: TestCase
id: TC-TRS-META-011
name: "Tagged-value multiplicity makes a value required or a list"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_typed.rs
verifies:
  - REQ-TRS-META-011
tags:
  - metadata
---

```gherkin
Feature: Tagged-value multiplicity makes a value required or a list

  Scenario: required value omitted
    Given a feature with multiplicity 1 and no default
    When an application omits it
    Then E325 is reported, also on a draft element

  Scenario: too many values
    Given a feature with multiplicity 0..1
    When an application gives a list of two
    Then E325 is reported

  Scenario: list accepted
    Given a feature with multiplicity 1..*
    When an application gives a YAML list of two values
    Then no finding is reported

  Scenario: default multiplicity
    Given a feature with no multiplicity
    When an application omits it
    Then no finding is reported
```
