---
type: TestCase
id: TC-TRS-META-063
name: "MCP gates tool evaluates the project's stereotype gates"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_stereotypes.rs
verifies:
  - REQ-TRS-META-063
tags:
  - metadata
  - mcp
---

```gherkin
Feature: MCP gates tool evaluates the project's stereotype gates

  Scenario: deny_meta and coverage
    Then tripped gates are listed

  Scenario: none configured
    Then tripped is false

  Scenario: malformed
    Then it is a tool error
```
