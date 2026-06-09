# show — element details and documentation

## SYNOPSIS
    syscribe -m <root> show <qname|id>

## DESCRIPTION
Prints an element's frontmatter fields (type, status, integrity level, extRef,
domain, …), its inline features, and its documentation body. Accepts a qualified
name (Pkg::Sub::Name) or a stable id (REQ-*, TC-*, SG-*, …).

## EXAMPLES
    syscribe -m model/ show UAV::Avionics::FlightController
    syscribe -m model/ show REQ-UAV-NAV-001

## SEE ALSO
    links, refs, trace, find
