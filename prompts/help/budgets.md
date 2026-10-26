# budgets — memory use against capacity per hardware part

## SYNOPSIS
    syscribe -m <root> budgets [--json]

## DESCRIPTION
For every hardware `PartDef`/`Part` that declares a capacity (`ramCapacity`, `flashCapacity`,
`nvmCapacity`), reports each resource: what is used, the capacity, the margin, the limit
(`capacity × margin / 100`), the utilisation and the largest contributors. The contributors are the
software components allocated to the part (`allocatedTo:`, `allocatedFrom:` or an `Allocation`) and
everything they contain, each counted once; RAM includes `stack`. Sizes are a number of bytes or text
with a unit (`B`, `kB`/`MB`/`GB`, `KiB`/`MiB`/`GiB`). The margin is the part's own `margin`, else
`[budgets] margin` in `.syscribe.toml`, else 100. The same computation backs `E690` (use above the
limit) and `E691` (a malformed size or margin) in `validate`. A hardware part that declares `cpuMargin` and/or `scheduling` is also checked for CPU
utilisation: its tasks (software parts with a `period` and a `wcet` duration) sum `wcet / period` against the
bound (1 for `edf`, Liu–Layland for `rate-monotonic`) × `cpuMargin` (`E692`); a `wcet` above a `deadline` is
`E693`. Always exits 0.

## OPTIONS
    --json   {budgets: [{element, resource, usedBytes, capacityBytes, marginPercent, limitBytes, utilisation,
             contributors: [{element, bytes}]}], cpu: [{element, scheduling, marginPercent,
             utilisation, boundPercent, limitPercent, tasks: [{element, wcetSeconds, periodSeconds, utilisation}]}]}

## EXAMPLES
    syscribe -m model/ budgets
    syscribe -m model_gti/ budgets --json

## SEE ALSO
    validate, meta
