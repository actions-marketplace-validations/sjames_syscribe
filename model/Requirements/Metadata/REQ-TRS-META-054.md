---
type: Requirement
id: REQ-TRS-META-054
name: "A baseline can require a stereotype condition"
status: draft
reqDomain: software
reqClass: system
tags:
  - metadata
---

`baseline create` shall refuse to seal while a stated stereotype condition is violated (GH #379).

## Behavior

- A condition such as no unverified `Regulatory` blocks sealing, naming the offending elements; `--force` is not provided.
