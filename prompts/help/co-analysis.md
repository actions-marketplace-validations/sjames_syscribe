# co-analysis — safety ↔ security co-engineering view

## SYNOPSIS
    syscribe -m <root> co-analysis [--json]

## DESCRIPTION
Shows the cross-domain overlap (ISO 26262 ⇄ ISO/SAE 21434): for each SafetyGoal /
HazardousEvent, the cyber threats that can violate it — following
ThreatScenario → damageScenarios → DamageScenario → hazardRef → goal — plus a
"gaps" section of safety-tagged DamageScenarios with no hazardRef (W030).

## OPTIONS
    --json   Emit {goals:[…], unlinkedSafetyDamage:[…]}.

## EXAMPLES
    syscribe -m model/ co-analysis
    syscribe -m model/ co-analysis --json

## SEE ALSO
    cyber-risk, safety-case, validate (W030), spec safety
