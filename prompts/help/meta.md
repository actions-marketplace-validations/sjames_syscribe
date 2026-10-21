# meta — report the elements that apply a stereotype, grouped by a tagged value

## SYNOPSIS
    syscribe -m <root> meta [--def <Def>] [--group-by <key>] [--json]

## DESCRIPTION
Without `--def`, counts the applications of every stereotype (`MetadataDef`) in the model.
With `--def <Def>` (a qualified name or last segment), lists the elements that apply `Def` or a
specialisation of it. With `--group-by <key>` the elements are grouped by that tagged value —
written or defaulted — largest group first; an element that does not give the value is in the
`(unset)` group (`null` in JSON). This is the compliance worklist: `Regulatory` grouped by
`status`.

## OPTIONS
    --def <Def>      The stereotype to report. An unknown name is a usage error (exit 1).
    --group-by <k>   Group by the tagged value `k`.
    --json           {definitions:[{def,count}]} · {def,count,elements} · {def,groupBy,count,groups:[{value,count,elements}]}.

## EXAMPLES
    syscribe -m model/ meta
    syscribe -m model/ meta --def DALAnnotation --group-by level

## SEE ALSO
    list --where meta.<Def>.<key>, validate --deny-meta
