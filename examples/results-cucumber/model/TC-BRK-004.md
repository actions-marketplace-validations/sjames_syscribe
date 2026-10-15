---
id: TC-BRK-004
type: TestCase
name: "Parking brake"
status: active
testLevel: L4
verifies: [REQ-BRK-001]
extRef: "TAP:parking-brake"
---

```gherkin
Feature: parking brake

  Scenario: Parking brake holds
    When the parking brake is applied
    Then the vehicle does not move
```
