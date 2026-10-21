---
type: TestCase
id: TC-TRS-META-031
name: "Stereotypes ship as profiles enabled in configuration"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_profiles.rs
verifies:
  - REQ-TRS-META-031
tags:
  - metadata
---

```gherkin
Feature: Stereotypes ship as profiles enabled in configuration

  Scenario: enabled
    Given a package with profile: true and [profiles.safety] in .syscribe.toml
    Then its definitions' checks are active

  Scenario: disabled
    Then the model is unchanged and the definitions remain usable by reference

  Scenario: version
    Then the profile version is reported by list
```
