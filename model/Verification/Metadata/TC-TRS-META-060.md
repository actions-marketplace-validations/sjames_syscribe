---
type: TestCase
id: TC-TRS-META-060
name: "MCP get_element returns the resolved stereotype applications"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/mcp_stereotypes.rs
verifies:
  - REQ-TRS-META-060
tags:
  - metadata
  - mcp
---

```gherkin
Feature: MCP get_element returns the resolved stereotype applications

  Scenario: applications with defaults
    Then get_element carries stereotypes with def, values and defaulted

  Scenario: specialisation and derived
    Then inherited defaults and derived applications are included

  Scenario: none
    Then the key is absent
```
