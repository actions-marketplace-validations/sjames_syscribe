---
type: TestCase
id: TC-TRS-META-061
name: "MCP search and list_by_type filter by stereotype"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_stereotypes.rs
verifies:
  - REQ-TRS-META-061
tags:
  - metadata
  - mcp
---

```gherkin
Feature: MCP search and list_by_type filter by stereotype

  Scenario: clauses
    Then all, any, negation and specialisation behave as in the CLI

  Scenario: unknown name
    Then it is a tool error

  Scenario: list_by_type where and paging
    Then where filters, and total counts the filtered set
```
