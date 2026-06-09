# links — all outbound and inbound relationships

## SYNOPSIS
    syscribe -m <root> links <qname|id>

## DESCRIPTION
Lists every relationship touching an element in both directions — outbound
(supertype, typedBy, satisfies, verifies, implementedBy, …) and inbound (what
references it). Useful for impact analysis before editing.

## EXAMPLES
    syscribe -m model/ links UAV::Airframe

## SEE ALSO
    refs, connectivity, trace
