---
type: TestCase
id: TC-TRS-META-041
name: "A stereotype can be a diagram grouping criterion"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/vis_stereotype.rs
verifies:
  - REQ-TRS-META-041
tags:
  - metadata
---

```gherkin
Feature: A stereotype can be a diagram grouping criterion

  Scenario: swimlane
    Given a frame style on External
    Then elements applying it are grouped in a swimlane

  Scenario: specialisation
    Then sub-definition carriers are in the same lane
```
