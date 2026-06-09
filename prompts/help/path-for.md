# path-for — file path for an element

## SYNOPSIS
    syscribe -m <root> path-for <qname|id>

## DESCRIPTION
Prints the on-disk file path for an element, given its qualified name or stable id
— handy for editor/script integration.

## EXAMPLES
    syscribe -m model/ path-for UAV::Avionics::FlightController
    syscribe -m model/ path-for REQ-UAV-NAV-001

## SEE ALSO
    show
