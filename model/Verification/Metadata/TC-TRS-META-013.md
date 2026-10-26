---
type: TestCase
id: TC-TRS-META-013
name: "Tagged values may reference elements"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_typed.rs
verifies:
  - REQ-TRS-META-013
tags:
  - metadata
---

```gherkin
Feature: Tagged values may reference elements

  Scenario: resolving reference
    Given reviewer is a reference feature
    When an application sets reviewer: User::jane and it resolves
    Then no finding is reported

  Scenario: unresolved reference
    When it does not resolve
    Then E326 is reported

  Scenario: wrong target type
    When it resolves to an element type the feature does not allow
    Then E326 is reported

  Scenario: traced
    Then refs, impact and links list the reference with the stereotype and key
    And deleting the target is blocked as a referrer
```
