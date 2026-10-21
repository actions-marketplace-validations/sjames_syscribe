---
type: Requirement
id: REQ-TRS-ARCH-008
name: "rule waivers"
status: draft
reqDomain: software
reqClass: system
tags:
  - architecture-rules
---

A violation of an `ArchitectureRule` shall be waivable with a recorded reason and an expiry (GH #369).

## Behavior

- `waivers:` is a list of `{element, rationale, expires}`. A finding whose subject is the waived element is suppressed while the waiver is current. The subject is the dependent component for `forbid-dependency`, `allow-only`, `layering` and `max-depth`, and the shared component for `independence`.
- `expires` is a date `YYYY-MM-DD`; on or before today the waiver no longer applies, the finding returns, and `W664` names the waiver as expired. A waiver that suppresses nothing is also `W664`.
- A waiver without `element`, `rationale` or a valid `expires` is `E662`.
