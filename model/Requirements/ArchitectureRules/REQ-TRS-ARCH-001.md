---
type: Requirement
id: REQ-TRS-ARCH-001
name: "An ArchitectureRule is a model element"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

A rule over the architecture shall be a first-class model element evaluated by `validate` (GH #369).

## Behavior

- `type: ArchitectureRule`, id `AR-…` (pattern `^AR(-[A-Z0-9]{2,12})*-[0-9]{3,8}$`, additional prefixes through `[ids.prefixes]`), `name` required, files `<id>.md`.
- Fields: `kind` (`forbid-dependency` or `independence`), `from` and `to` (selectors, REQ-TRS-ARCH-002), `via` (a non-empty subset of `connections`, `typedBy`, `allocations`; default all three), `severity` (`error` default, or `warning`), `rationale`.
- A rule with `status: draft` is not evaluated.
- A malformed rule — unknown `kind`, a missing `from`/`to`, a bad `via` or `severity`, a malformed selector — is `E662` on the rule and is not evaluated.
- `kind`, `from`, `to` and `via` on any other element type are `W047`.
