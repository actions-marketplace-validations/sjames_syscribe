---
id: TC-BRK-001
type: TestCase
name: "Brake request timing on the rigs"
status: active
testLevel: L4
verifies: [REQ-BRK-001]
---

```gherkin
Feature: brake request timing

  Scenario: Request reaches the actuator in time
    Given the vehicle is stationary
    When a brake request is sent
    Then the actuator receives it within 5 ms

  Scenario: Request during a bus fault is not lost
    Given a CAN bus fault is injected
    When a brake request is sent
    Then the actuator still receives it
```
