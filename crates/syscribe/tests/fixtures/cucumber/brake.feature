@feature-tag
Feature: Brake request timing

  Background:
    Given the vehicle is stationary

  @TC-BRK-001
  Scenario: Request reaches the actuator in time
    When a brake request is sent
    Then the actuator receives it within 5 ms

  @TC-BRK-001
  Scenario: Request during a bus fault is not lost
    When a CAN bus fault is injected
    And a brake request is sent
    Then the actuator still receives it

  @extref:TAP:parking-brake
  Scenario: Parking brake holds
    When the parking brake is applied
    Then the vehicle does not move

  @TC-BRK-002
  Scenario: A pending one
    When a step is pending
    Then nothing else matters

  @TC-BRK-002
  Scenario: An undefined one
    When a step has no definition

  @TC-BRK-003
  Scenario Outline: Latency by speed
    When a brake request is sent at <speed> km/h
    Then the latency is below <limit> ms

    Examples:
      | speed | limit |
      | 30    | 5     |
      | 60    | 5     |
      | 120   | 2     |

  @TC-BRK-003
  Scenario: Flaky one
    When a flaky step runs

  @TC-BRK-002 @hookfail
  Scenario: Hook failure
    When a brake request is sent

  Scenario: Untagged scenario
    When a brake request is sent
