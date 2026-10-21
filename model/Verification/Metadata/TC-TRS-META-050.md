---
type: TestCase
id: TC-TRS-META-050
name: "A MetadataDef declares checks run on every carrier"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_checks.rs
verifies:
  - REQ-TRS-META-050
tags:
  - metadata
---

```gherkin
Feature: A MetadataDef declares checks run on every carrier

  Scenario: failing check
    Given Regulatory with the check status == "verified" or reviewer != ""
    When one element has status unverified and no reviewer
    Then E650 names the definition and that element only

  Scenario: warning severity
    Then a W650 check is suppressed on a draft element

  Scenario: inherited
    Then a sub-definition application runs the parent's checks

  Scenario: profile disabled
    Then checks of a disabled profile do not run
```
