# why — what requirements an element satisfies

## SYNOPSIS
    syscribe -m <root> why <qname>

## DESCRIPTION
Reports the requirements a given architecture element satisfies (its `satisfies:`
targets), the upstream side of the trace.

## EXAMPLES
    syscribe -m model/ why UAV::Avionics::FlightController

## SEE ALSO
    trace, who-verifies, links
