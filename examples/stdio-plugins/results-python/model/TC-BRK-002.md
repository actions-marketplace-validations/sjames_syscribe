---
id: TC-BRK-002
type: TestCase
name: "Parking brake holds"
status: active
testLevel: L4
verifies: [REQ-BRK-001]
extRef: "TAP:parking-brake"
---

```gherkin
Feature: parking brake

  Scenario: Parking brake holds on a slope
    Given the vehicle is on a 10 percent slope
    When the parking brake is applied
    Then the vehicle does not move
```
