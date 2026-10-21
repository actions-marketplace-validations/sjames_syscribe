---
type: TestCase
id: TC-TRS-META-042
name: "Edge stereotypes style connections"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/vis_stereotype_frames.rs
verifies:
  - REQ-TRS-META-042
tags:
  - metadata
---

```gherkin
Feature: Edge stereotypes style connections

  Scenario: styled edge
    Given a style on a stereotype applied to a Connection
    Then the edge is drawn with the configured colour and dash
```
