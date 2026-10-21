---
type: TestCase
id: TC-TRS-META-062
name: "MCP meta tool reports stereotype applications"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_stereotypes.rs
verifies:
  - REQ-TRS-META-062
tags:
  - metadata
  - mcp
---

```gherkin
Feature: MCP meta tool reports stereotype applications

  Scenario: grouped
    Then the JSON equals the CLI report

  Scenario: unknown def
    Then it is a tool error
```
