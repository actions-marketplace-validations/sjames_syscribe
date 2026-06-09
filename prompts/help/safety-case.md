# safety-case — GSN goal → argument → evidence tree

## SYNOPSIS
    syscribe -m <root> safety-case [<SG-id>] [--json]

## DESCRIPTION
Renders the assurance argument for each SafetyGoal (or the one given): the
Argument nodes (claim/strategy/solution) that support it and their evidence
(Requirements, TestCases with ingested verdicts, sub-Arguments, AssumptionOfUse).
It also folds in the implicit SafetyGoal ← Requirement (derivedFromSafetyGoal) ←
TestCase (verifies) chain, so it is useful even without explicit Argument nodes.

## OPTIONS
    --json   Emit {goals:[{id,title,arguments,requirements,assumptions}]}.

## EXAMPLES
    syscribe -m model/ safety-case
    syscribe -m model/ safety-case SG-BRAKE-001 --json

## SEE ALSO
    trace, co-analysis, spec safety
