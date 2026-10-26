---
type: Requirement
id: REQ-TRS-BADR-001
name: "The breakdown ADR requirement can be disabled"
status: draft
reqDomain: software
reqClass: system
tags:
  - traceability
---

A project shall be able to turn off the rule that every `derivedFrom:` requirement cites a `breakdownAdr:` (§12.2).

## Behavior

- `[traceability] require_breakdown_adr = false` in `.syscribe.toml` stops `E310` being raised. The default is `true`, so a model without the table behaves as before.
- A `breakdownAdr:` that is present is still checked: `E311` (does not resolve to an ADR) and `W303` (the ADR is still `proposed`) are unchanged.
- A value that is not a boolean is `W632` and the default applies.
