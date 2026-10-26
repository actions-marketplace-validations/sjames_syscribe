---
type: TestCase
id: TC-TRS-META-030
name: "A MetadataDef can specialise another"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_profiles.rs
verifies:
  - REQ-TRS-META-030
tags:
  - metadata
---

```gherkin
Feature: A MetadataDef can specialise another

  Scenario: queries and checks follow the supertype
    Given ASIL_D with supertype Safety
    Then list --metadata Safety finds its applications and Safety checks run on them

  Scenario: inherited values validate
    Then values inherited from Safety are type-checked on an ASIL_D application

  Scenario: widening
    When a sub-definition widens a range or enumeration
    Then E329 is reported

  Scenario: annotates narrows
    Then a sub-definition annotating a kind the parent does not is reported

  Scenario: abstract
    When an abstract definition is applied
    Then E328 is reported

  Scenario: supertype problems
    Then an unresolved, non-MetadataDef or cyclic supertype is E329
```
