# validate — report findings and gate CI

## SYNOPSIS
    syscribe -m <root> validate [--file <path>] [--json]
        [--deny <CODES>] [--max-warnings <N>] [--warnings-as-errors]
        [--profile <name>] [--config <C>] [--all-configs]
        [--results <file>] [--fetch-remote]

## DESCRIPTION
Runs the full per-element validation pass and prints the findings (errors and
warnings) only. By default a model with warnings but no errors exits 0; the
gating flags promote chosen warnings to build failures.

## OPTIONS
    --file <path>           Validate (and report) a single file only.
    --json                  Emit findings as a JSON array.
    --deny <CODES>          Comma-separated warning codes treated as gate failures.
    --max-warnings <N>      Fail when the warning count exceeds N.
    --warnings-as-errors    Treat every warning as a gate failure.
    --profile <name>        Apply a named [profiles.<name>] policy from .syscribe.toml
                            (SIL/ASIL-scopable code promotion). See `help` for profiles.
    --config <C>            Project onto a Configuration (id/qname or 'Features::A,Features::B')
                            and validate that variant (escaping refs: E226/W019).
    --all-configs           Validate every stored Configuration (CI gate).
    --results <file>        Ingest test results for this run (enables W010), no sidecar write.
    --fetch-remote          Run the .syscribe.toml [remote] hook to fetch remote sourceFiles.

## EXAMPLES
    # Findings table for the whole model
    syscribe -m model/ validate

    # CI gate: fail the build on coverage drift in safety areas
    syscribe -m model/ validate --deny W015,W300,W306

    # Apply a named policy from .syscribe.toml
    syscribe -m model/ validate --profile safety

    # Certify one product variant
    syscribe -m model/ validate --config CONF-UAV-DELIVERY-001

## EXIT CODES
    0  no errors (and no gate tripped)
    1  one or more Error-severity findings (errors always dominate)
    2  a warning tripped a gate (--deny / --max-warnings / --warnings-as-errors / --profile)

## SEE ALSO
    audit, verification-depth, matrix, spec validation
