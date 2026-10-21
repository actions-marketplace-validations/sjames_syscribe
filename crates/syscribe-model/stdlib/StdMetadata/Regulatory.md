---
type: MetadataDef
name: Regulatory
features:
  - name: status
    typedBy: ScalarValues::String
    enumValues: [verified, unverified, waived]
  - name: source
    typedBy: ScalarValues::String
  - name: reviewer
    typedBy: ScalarValues::String
---

A regulatory value and its verification status; `source` cites the regulation and `reviewer` who checked it.
