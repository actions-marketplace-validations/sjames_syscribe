*** Settings ***
Suite Setup    Log    suite setup

*** Test Cases ***
Request reaches the actuator in time
    [Tags]    TC-BRK-001    REQ-BRK-001    smoke
    [Documentation]    Checks latency
    Log    ok

Request during a bus fault is not lost
    [Tags]    TC-BRK-001
    Fail    message dropped after fault

Parking brake holds
    [Tags]    extref:TAP:parking-brake
    Log    ok

A skipped one
    [Tags]    TC-BRK-002
    Skip    rig busy

Untagged test
    Log    nothing joins this

Not a testcase tag
    [Tags]    REQ-BRK-001    smoke
    Log    only requirement tags
