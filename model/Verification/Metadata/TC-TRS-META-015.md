---
type: TestCase
id: TC-TRS-META-015
name: "Tagged values may be variant parameters"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_typed_cli.rs
verifies:
  - REQ-TRS-META-015
tags:
  - metadata
---

```gherkin
Feature: Tagged values may be variant parameters

  Scenario: substitution under a configuration
    Given a value {{Features::GridProfile.nominalFrequencyHz}}
    When validated with --config selecting 50
    Then the value 50 is type-, unit- and range-checked

  Scenario: no configuration
    Then the placeholder is left as written and not type-checked

  Scenario: unknown parameter
    Then the E203 family finding is reported
```
