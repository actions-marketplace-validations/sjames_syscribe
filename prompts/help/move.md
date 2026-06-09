# move — rename/relocate an element, rewriting references

## SYNOPSIS
    syscribe -m <root> move <src> <dest> [--dry-run]

## DESCRIPTION
Moves an element or package to a new qualified name, rewriting every cross-
reference to it across the model. Use --dry-run first to preview the changes.

## OPTIONS
    --dry-run   Show what would change without writing.

## EXAMPLES
    syscribe -m model/ move UAV::Avionics::FC UAV::Avionics::FlightController --dry-run
    syscribe -m model/ move UAV::Old UAV::New

## SEE ALSO
    refs, links
