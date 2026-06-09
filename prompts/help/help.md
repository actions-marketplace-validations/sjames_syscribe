# help — detailed per-command help

## SYNOPSIS
    syscribe help [<command>]
    syscribe <command> --help | -h

## DESCRIPTION
With no argument, lists every command with a one-line summary. With a command,
prints its detailed man page (synopsis, options, examples, exit codes, see-also).
`syscribe <command> --help` (or -h) prints the same page. These work without a
model directory.

## EXAMPLES
    syscribe help
    syscribe help validate
    syscribe validate --help
    syscribe matrix -h

## SEE ALSO
    spec, --agent-instructions
