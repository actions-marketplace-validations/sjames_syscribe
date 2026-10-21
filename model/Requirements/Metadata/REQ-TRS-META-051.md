---
type: Requirement
id: REQ-TRS-META-051
name: "validate --deny-meta gates on tagged values"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

`validate --deny-meta` and a `[gates]` table shall fail CI on a stereotype condition (GH #379).

## Behavior

- `validate --deny-meta Regulatory.status=unverified` (repeatable; any `list --where meta.…` form without the prefix) exits 2 — a tripped gate — exactly when a matching element exists, and lists them. A malformed condition is a usage error (exit 1).
- `[gates] deny_meta = ["…"]` in `.syscribe.toml` holds the same conditions as standing ones.
- Coverage-based gates (every `Safety` with `integrity>=2` verified) are `[[gates.coverage]]` of REQ-TRS-META-053.
