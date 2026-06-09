# safety-case — GSN goal → argument → evidence tree

## SYNOPSIS
    syscribe -m <root> safety-case [<SG-id>] [--config <C>] [--json]

## DESCRIPTION
Renders the assurance argument for each SafetyGoal (or the one given): the
Argument nodes (claim/strategy/solution) that support it and their evidence
(Requirements, TestCases with ingested verdicts, sub-Arguments, AssumptionOfUse).
It also folds in the implicit SafetyGoal ← Requirement (derivedFromSafetyGoal) ←
TestCase (verifies) chain, so it is useful even without explicit Argument nodes.

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   only goals and evidence active in that variant are assembled.
    --json         Emit {goals:[{id,title,arguments,requirements,assumptions}]}.

## EXAMPLES
    syscribe -m model/ safety-case
    syscribe -m model/ safety-case SG-BRAKE-001 --json
    syscribe -m model/ safety-case --config CONF-UAV-DELIVERY-001   # variant-scoped

## SEE ALSO
    trace, co-analysis, spec safety
