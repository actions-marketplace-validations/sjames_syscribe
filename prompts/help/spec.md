# spec — browse the embedded format specification

## SYNOPSIS
    syscribe spec [<section>]

## DESCRIPTION
Prints the embedded Syscribe format reference (no model needed). Sections:
toc (default), types, fields, namespace, validation, traceability, safety. Use
this to look up element types, every frontmatter field, and every rule code.

## EXAMPLES
    syscribe spec types
    syscribe spec fields
    syscribe spec safety
    syscribe spec validation

## SEE ALSO
    help, template, --agent-instructions
