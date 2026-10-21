---
type: TestCase
id: TC-TRS-META-044
name: "All renderers agree on stereotype style"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/metadata_style.rs
verifies:
  - REQ-TRS-META-044
tags:
  - metadata
---

```gherkin
Feature: All renderers agree on stereotype style

  Scenario: one style, all renderers
    Then the browser model, static SVG and Mermaid/PlantUML all carry the same resolved style

  Scenario: degraded property
    Then an inexpressible property degrades to the nearest available
```
