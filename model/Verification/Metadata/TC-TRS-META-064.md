---
type: TestCase
id: TC-TRS-META-064
name: "MCP write tools refuse the built-in stereotype library"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_stereotypes.rs
verifies:
  - REQ-TRS-META-064
tags:
  - metadata
  - mcp
---

```gherkin
Feature: MCP write tools refuse the built-in stereotype library

  Scenario: each write tool
    Then StdMetadata is refused with a read-only message and nothing is written
```
