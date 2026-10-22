# design-report — the detailed design of one software part

## SYNOPSIS
    syscribe -m <root> design-report <part> [--json]

## DESCRIPTION
Lays out one software `PartDef`/`Part` (`domain: software`) as the detailed-design work product
(ASPICE SWE.3). Sections: the part's `level` and description; its **interface** (its own `operations:` and
those of the ports and interfaces it contains, with parameters and return type); its **behaviour** (the
`ActionDef`s that realise its operations); its **data** (its other `features:`, with type, unit and the
`range` of an `AttributeDef`); its **resources** (`ram`, `flash`, `nvm`, `stack`, `period`, `deadline`,
`wcet`, `priority` as declared); its **code** (`implementedBy:`); the **requirements** it satisfies; and its
**tests** (TestCases that verify it or a requirement it satisfies, with level and verdict from ingested
results). A section with nothing to show says `None.`. An element that is not a software part is a usage
error (exit 1).

## OPTIONS
    --json   {element, level, description, interface: [{owner, name, parameters, returnType, isAsync,
             realizedBy}], behaviour, data: [{name, type, unit, range}], resources, code, requirements,
             tests: [{id, level, verdict}]}

## EXAMPLES
    syscribe -m model/ design-report UAV::Avionics::FlightController
    syscribe -m model/ design-report UAV::Avionics::FlightController --json

## SEE ALSO
    show, budgets, validate
