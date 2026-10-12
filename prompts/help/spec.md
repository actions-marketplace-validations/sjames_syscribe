# spec — browse the embedded format specification

## SYNOPSIS
    syscribe spec [<section>]

## DESCRIPTION
Prints the embedded Syscribe format reference (no model needed). Sections:
toc (default), types, fields, namespace, validation, traceability, safety, behaviour,
evidence, diagrams, variability. Use this to look up element types, every frontmatter
field, every rule code, and the guides for executable models, evidence, diagrams and
product lines (the last four print the model-guide pages themselves).

## EXAMPLES
    syscribe spec types
    syscribe spec fields
    syscribe spec safety
    syscribe spec validation
    syscribe spec behaviour
    syscribe spec evidence

## SEE ALSO
    help, template, --agent-instructions
