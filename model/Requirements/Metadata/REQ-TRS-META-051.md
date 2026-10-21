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

- `validate --deny-meta Regulatory.status=unverified` exits non-zero exactly when a matching element exists, and lists them.
- `[gates]` in `.syscribe.toml` holds the same conditions for CI.
- Coverage-based gates (e.g. every `Safety` with `integrity>=2` covered by a campaign) use the selector grammar.
