# cyber-risk — ISO/SAE 21434 risk determination

## SYNOPSIS
    syscribe -m <root> cyber-risk [--json]

## DESCRIPTION
Lists each ThreatScenario with its computed risk: severity (max damageSeverity
over its damageScenarios) × attackFeasibility → low/medium/high/critical, its
riskTreatment, whether it is addressed by a CybersecurityGoal, and a flag
(untreated when it would trip W031).

## OPTIONS
    --json   Emit {id, severity, feasibility, risk, treatment, addressed, flag} array.

## EXAMPLES
    syscribe -m model/ cyber-risk
    syscribe -m model/ cyber-risk --json

## NOTES
An untreated high/critical threat raises W031, and a CybersecurityGoal with a
CAL below its threats' risk raises W032, both in `validate`.

## SEE ALSO
    co-analysis, validate (W031/W032), spec safety
