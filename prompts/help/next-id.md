# next-id — next available stable ID for a prefix

## SYNOPSIS
    syscribe -m <root> next-id <id-prefix>

## DESCRIPTION
Prints the next non-colliding stable ID for a prefix (Requirement, TestCase, ADR,
and the safety/security id families), so a new element gets a unique id.

## EXAMPLES
    syscribe -m model/ next-id REQ-UAV-NAV
    syscribe -m model/ next-id TC-SCHED

## SEE ALSO
    template, check-ref
