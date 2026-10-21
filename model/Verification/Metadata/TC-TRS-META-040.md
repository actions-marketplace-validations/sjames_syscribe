---
type: TestCase
id: TC-TRS-META-040
name: "Diagram style is driven by stereotypes"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/vis_stereotype.rs
verifies:
  - REQ-TRS-META-040
tags:
  - metadata
---

```gherkin
Feature: Diagram style is driven by stereotypes

  Scenario: configured outline
    Given a style mapping SafetyCritical to a red stroke
    Then an element with the stereotype is drawn with that stroke in the SVG

  Scenario: no style
    When the style is removed
    Then the output equals today's output byte for byte

  Scenario: condition
    Given a style with integrity>=2
    Then it applies only to elements where that is true

  Scenario: precedence
    Given two stereotypes setting the same property
    Then the documented precedence decides
```
