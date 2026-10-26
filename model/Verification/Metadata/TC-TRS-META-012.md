---
type: TestCase
id: TC-TRS-META-012
name: "Tagged-value defaults are visible without being written"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/metadata_typed_cli.rs
verifies:
  - REQ-TRS-META-012
tags:
  - metadata
---

```gherkin
Feature: Tagged-value defaults are visible without being written

  Scenario: default in show
    Given a feature with default 2
    When an application omits it
    Then show and show --json report 2 marked defaulted

  Scenario: default in list --where
    Then list --where meta.Def.key=2 matches that element without the file changing

  Scenario: explicit value wins
    When the application gives 3
    Then 3 is reported

  Scenario: default violating its own type
    Given a default of "x" on an Integer feature
    Then E322 is reported on the definition

  Scenario: required feature with default
    Then omitting it is not E325
```
