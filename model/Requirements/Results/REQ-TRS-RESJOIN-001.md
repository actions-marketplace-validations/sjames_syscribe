---
type: Requirement
id: REQ-TRS-RESJOIN-001
name: "A result is joined to its TestCase by id or extRef, and a result older than a change to its test or requirement is reported"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
  - traceability
---

A result shall be attached to the TestCase it is about by a stable key a test tool can know, and a verdict shall not silently outlive a change to the test or to what the test verifies.

## Behavior

- **Join key.** The `testCase` of a record is resolved in this order: a TestCase id; a qualified name; an `extRef` value of a TestCase (`extRef` is already first-class and `W028` already flags the same value on two elements, so it is unique per model). A reference that matches more than one element, or matches an element that is not a TestCase, is an error naming the candidates.
- **Unmatched records are reported, not dropped.** `ingest-results` prints the number of records it could not join and the first few keys. By default the matched records are stored; with `--strict` any unmatched record rejects the whole ingest and nothing is written (exit 1). Unmatched records are kept under `unmatched` in the sidecar so `results failures` can list them.
- **Content hashes.** At ingest each stored record keeps `tcHash`, the hash of the TestCase's normative projection, and `reqHashes`, the same for each element it `verifies:`, using the hashing of suspect links (`ADR-SYS-SUSLINK-001`, `element_hash`) with its editorial exclusions.
- **Stale results.** `validate` recomputes the hashes. If the TestCase or a verified requirement changed since the verdict was recorded, a new warning (code allocated at implementation, in the family of `W090`) is raised on the TestCase, naming what changed; it is gateable with `--deny`. A record ingested without hashes (an older sidecar) is silent. The verdict itself is not altered.
- **Re-baselining.** Re-ingesting the results of the changed test replaces the record and its hashes. There is no `accept` command for results: a result is evidence of a run, not a review.
- **Interaction with the rest.** Stale does not mean failing; `trace` and `matrix` annotate such a verdict as `[stale]`. A baseline that seals a TestCase does not seal results.

## Documentation and spec (part of the acceptance of this requirement)

The `extRef` rows of `docs/format/frontmatter.md` and the spec (now also the result join key); the `TestCase` skeleton printed by `syscribe template TestCase` (a commented `extRef:` line explaining the key) and its test; `docs/design/suspect-links.md` (cross reference); the help pages `ingest-results`, `results` and `validate`; both validation catalogues; `docs/model-guide/executable-evidence.md`; and the drift tests.
