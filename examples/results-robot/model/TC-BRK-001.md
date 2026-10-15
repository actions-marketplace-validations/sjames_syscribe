---
id: TC-BRK-001
type: TestCase
testLevel: L5
status: active
name: T
verifies: [REQ-BRK-001]
---

```gherkin
Feature: f
  Scenario: Request reaches the actuator in time
    Given x
    Then y

  Scenario: Request during a bus fault is not lost
    Given x
    Then y
```
