# check-ref — verify a cross-reference resolves

## SYNOPSIS
    syscribe -m <root> check-ref <qname|id>

## DESCRIPTION
Verifies that a cross-reference resolves to a known element and prints its type —
use it before writing a supertype/typedBy/verifies/derivedFrom value.

## EXAMPLES
    syscribe -m model/ check-ref Interfaces::PowerPortDef
    syscribe -m model/ check-ref REQ-UAV-NAV-001

## SEE ALSO
    template, next-id, links
