# feature-check — holistic feature-model validation

## SYNOPSIS
    syscribe -m <root> feature-check [--json] [--deny <CODES>]
        [--deep] [--count] [--enumerate] [--prove <dir>]

## DESCRIPTION
Validates the feature model as a whole (separate from `validate`): requires/
excludes resolution and satisfaction, dead/always-on optional features, circular
derivedFrom, bindTo ranges, parameterConstraints, binding-time ordering, and
orphan features (W024). With --deep, runs SAT-backed analysis over the whole
configuration space.

## OPTIONS
    --deny <CODES>   Treat the listed warnings as gate failures.
    --deep           SAT analysis: void model, dead/core/false-optional features,
                     full-semantics configuration validity, with explanations.
    --count          Number of valid configurations.
    --enumerate      List the valid configurations.
    --prove <dir>    Write DIMACS CNF for each UNSAT finding.
    --json           Machine-readable output.

## EXAMPLES
    syscribe -m model/ feature-check
    syscribe -m model/ feature-check --deep
    syscribe -m model/ feature-check --count

## EXIT CODES
    0  no errors    1  errors present (dormant exit 0 with a notice if no feature model)

## SEE ALSO
    features, feature, configure, matrix --features, spec validation
