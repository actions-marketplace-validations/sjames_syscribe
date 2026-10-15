---
id: TC-BRK-003
type: TestCase
name: "Latency by speed"
status: active
testLevel: L4
verifies: [REQ-BRK-001]
---

```gherkin
Feature: latency

  Scenario Outline: Latency by speed
    When a brake request is sent at <speed> km/h
    Then the latency is below <limit> ms

    Examples:
      | speed | limit |
      | 30    | 5     |
```
