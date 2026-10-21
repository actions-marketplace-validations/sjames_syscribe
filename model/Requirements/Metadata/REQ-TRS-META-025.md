---
type: Requirement
id: REQ-TRS-META-025
name: "A stereotype application may be variant-dependent"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

A metadata application shall carry `appliesWhen:` and be projected with its element (GH #376).

## Behavior

- `appliesWhen:` on a map application uses the feature-expression language of `appliesWhen` on elements.
- Under `--config` an application whose guard is false is dropped from every consumer; without `--config` all applications are present.
- `diff --config A --config B` reports an application that differs between variants.
