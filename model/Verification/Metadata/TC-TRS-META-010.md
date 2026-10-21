---
type: TestCase
id: TC-TRS-META-010
name: "Tagged values are type-, unit-, range- and enumeration-checked"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_typed.rs
verifies:
  - REQ-TRS-META-010
tags:
  - metadata
---

```gherkin
Feature: Tagged values are type-, unit-, range- and enumeration-checked

  Scenario: wrong scalar type
    Given a MetadataDef Safety whose integrity is typed Integer
    When an element applies Safety with integrity "banana"
    Then E322 is reported on that element naming Safety and integrity

  Scenario: unit of a different dimension
    Given a feature declared with unit kg
    When a value is given in seconds
    Then E322 is reported

  Scenario: out of range
    Given integrity has range 0..4
    When an element applies integrity 7
    Then E323 is reported

  Scenario: not in enumeration
    Given status has enumValues [verified, unverified]
    When an element applies status "maybe"
    Then E324 is reported

  Scenario: undeclared typing is unchanged
    Given a feature with no typedBy, range or enumValues
    Then only W045 can apply and no E322, E323 or E324 is reported

  Scenario: bare reference application
    Given an element applies a definition by bare reference
    Then no new finding is reported
```
