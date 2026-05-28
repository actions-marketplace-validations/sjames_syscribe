# Tool Validation Report

**Tool:** syscribe CLI validator  
**Version:** unknown  
**Standard:** ISO 26262:2018 Part 8 §11 (TCL2), IEC 61508:2010 Part 3 Annex D  
**Date:** 2026-05-28  
**TRS:** `qual/Requirements/`  **Test cases:** `qual/TestCases/`

---

## 1. Summary

| Metric | Value |
|---|---|
| Total test cases | 48 |
| Passed | 48 |
| Failed | 0 |
| Overall verdict | **PASS** |

---

## 2. Results

### TC-TRS-CLI-001 — Verify that the tool accepts the model directory via -m and --model arguments.

**Verifies:** REQ-TRS-CLI-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| --model long form | ✓ PASS |

---

### TC-TRS-CLI-002 — Verify that the tool reports an error to stderr and exits non-zero for invalid model paths.

**Verifies:** REQ-TRS-CLI-002  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| non-existent path exits non-zero | ✓ PASS |

---

### TC-TRS-CLI-003 — Verify that --agent-instructions prints the LLM prompt and exits 0 without requiring -m.

**Verifies:** REQ-TRS-CLI-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| --agent-instructions prints prompt and exits 0 | ✓ PASS |

---

### TC-TRS-ELEM-001 — Verify that all element types defined in §2 are recognised and processed without E005.

**Verifies:** REQ-TRS-ELEM-001  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| all defined element types are recognised without E005 | ✓ PASS |

---

### TC-TRS-ELEM-002 — Verify that an unrecognised type: value produces exactly one E005 finding.

**Verifies:** REQ-TRS-ELEM-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| completely unknown type value produces E005 | ✓ PASS |
| wrong-case type value produces E005 | ✓ PASS |

---

### TC-TRS-ELEM-003 — Verify that implicit base library supertypes are applied when no supertype: is given.

**Verifies:** REQ-TRS-ELEM-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| PartDef with no supertype: loads without E004 | ✓ PASS |

---

### TC-TRS-ID-001 — Verify that Requirement elements are validated against the REQ-* id pattern.

**Verifies:** REQ-TRS-ID-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid REQ-* id patterns are accepted | ✓ PASS |
| invalid REQ-* id pattern produces E006 | ✓ PASS |

---

### TC-TRS-ID-002 — Verify that TestCase elements are validated against the TC-* id pattern.

**Verifies:** REQ-TRS-ID-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid TC-* id pattern is accepted | ✓ PASS |
| invalid TC-* id pattern produces E006 | ✓ PASS |

---

### TC-TRS-ID-003 — Verify that ADR elements are validated against the ADR-* id pattern.

**Verifies:** REQ-TRS-ID-003  
**Result:** ✓ PASS (7 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid ADR-* id is accepted | ✓ PASS |
| ADR id not matching pattern produces E300 | ✓ PASS |
| ADR missing id produces E301 | ✓ PASS |

---

### TC-TRS-ID-004 — Verify that duplicate id: values across elements produce E101.

**Verifies:** REQ-TRS-ID-004  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| two elements with the same id produce E101 | ✓ PASS |
| unique ids produce no E101 | ✓ PASS |

---

### TC-TRS-OUT-001 — Verify that the tool writes its validation report to stdout in Markdown format.

**Verifies:** REQ-TRS-OUT-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| report is written to stdout in Markdown format | ✓ PASS |

---

### TC-TRS-OUT-002 — Verify that each finding in the report contains severity, code, element reference, and description.

**Verifies:** REQ-TRS-OUT-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| findings table has Code, File, and Message columns | ✓ PASS |

---

### TC-TRS-OUT-003 — Verify that the report includes a summary section with error and warning counts.

**Verifies:** REQ-TRS-OUT-003  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| summary section includes error and warning counts | ✓ PASS |

---

### TC-TRS-OUT-004 — Verify that the tool exits non-zero when any Error-severity finding is present.

**Verifies:** REQ-TRS-OUT-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| model with errors exits non-zero | ✓ PASS |
| model with errors and warnings still exits non-zero | ✓ PASS |

---

### TC-TRS-OUT-005 — Verify that the tool exits with code 0 when no Error findings are present.

**Verifies:** REQ-TRS-OUT-005  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| model with only warnings exits 0 | ✓ PASS |
| clean model exits 0 | ✓ PASS |

---

### TC-TRS-PARSE-001 — Verify that the tool accepts a model root directory path and uses it as the namespace root.

**Verifies:** REQ-TRS-PARSE-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid directory is accepted | ✓ PASS |
| empty directory produces zero elements | ✓ PASS |

---

### TC-TRS-PARSE-002 — Verify that the tool recursively discovers .md files in nested subdirectories.

**Verifies:** REQ-TRS-PARSE-002  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| elements in nested directories are discovered | ✓ PASS |
| non-.md files are ignored | ✓ PASS |

---

### TC-TRS-PARSE-003 — Verify that standard build and tool directories are excluded from discovery.

**Verifies:** REQ-TRS-PARSE-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| target/ directory is excluded from discovery | ✓ PASS |

---

### TC-TRS-PARSE-004 — Verify that .sysmlignore patterns suppress file discovery.

**Verifies:** REQ-TRS-PARSE-004  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| .sysmlignore suppresses matching files | ✓ PASS |
| absence of .sysmlignore causes no error | ✓ PASS |

---

### TC-TRS-PARSE-005 — Verify that _index.md is treated as the package declaration for its directory.

**Verifies:** REQ-TRS-PARSE-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| _index.md is treated as package declaration | ✓ PASS |

---

### TC-TRS-PARSE-006 — Verify that a file with unparseable frontmatter produces a warning but does not halt processing.

**Verifies:** REQ-TRS-PARSE-006  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| malformed YAML frontmatter produces a warning, not a fatal error | ✓ PASS |

---

### TC-TRS-PARSE-007 — Verify that frontmatter is recognized only when the opening --- is the first line.

**Verifies:** REQ-TRS-PARSE-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with --- on first line is parsed correctly | ✓ PASS |
| file with blank first line produces E001 (missing frontmatter) | ✓ PASS |

---

### TC-TRS-PARSE-008 — Verify that invalid YAML frontmatter produces error E002.

**Verifies:** REQ-TRS-PARSE-008  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| valid YAML frontmatter produces no E002 | ✓ PASS |
| invalid YAML frontmatter produces E002 | ✓ PASS |

---

### TC-TRS-PARSE-009 — Verify that a file without a type: field is skipped with a warning.

**Verifies:** REQ-TRS-PARSE-009  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| file with no type: field is skipped with a warning | ✓ PASS |
| file with type: present is processed normally | ✓ PASS |

---

### TC-TRS-QNAME-001 — Verify that qualified names are derived correctly from directory path and filename stem.

**Verifies:** REQ-TRS-QNAME-001  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| single-level element has single-segment qualified name | ✓ PASS |
| three-level nested element has three-segment qualified name | ✓ PASS |

---

### TC-TRS-QNAME-002 — Verify that the name: field in _index.md overrides the directory name in qualified names.

**Verifies:** REQ-TRS-QNAME-002  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| name: in _index.md replaces directory name in qualified names | ✓ PASS |

---

### TC-TRS-QNAME-003 — Verify that the name: field in element frontmatter overrides the filename stem.

**Verifies:** REQ-TRS-QNAME-003  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| name: in frontmatter replaces the filename stem | ✓ PASS |

---

### TC-TRS-QNAME-004 — Verify that _index.md contributes no name segment to its package or sibling elements.

**Verifies:** REQ-TRS-QNAME-004  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| _index.md contributes no _index name segment | ✓ PASS |

---

### TC-TRS-TRACE-001 — Verify that computed reverse indices are populated from downstream link fields.

**Verifies:** REQ-TRS-TRACE-001  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| verifiedBy is computed from TestCase.verifies: | ✓ PASS |

---

### TC-TRS-TRACE-002 — Verify that E310 is emitted when derivedFrom: is present but breakdownAdr: is absent.

**Verifies:** REQ-TRS-TRACE-002  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| derivedFrom with no breakdownAdr produces E310 | ✓ PASS |
| derivedFrom with valid breakdownAdr produces no E310 | ✓ PASS |

---

### TC-TRS-TRACE-003 — Verify that W303 is emitted when a breakdownAdr: references a proposed ADR on an approved requirement.

**Verifies:** REQ-TRS-TRACE-003  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| approved requirement with proposed ADR produces W303 | ✓ PASS |

---

### TC-TRS-TRACE-004 — Verify that W300 is emitted for an approved leaf Requirement with no satisfying element.

**Verifies:** REQ-TRS-TRACE-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| approved leaf requirement with no satisfies produces W300 | ✓ PASS |
| approved leaf requirement with satisfies produces no W300 | ✓ PASS |

---

### TC-TRS-TRACE-005 — Verify that E312 is emitted when a parent Requirement appears in a satisfies: list.

**Verifies:** REQ-TRS-TRACE-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| architecture element satisfying a parent requirement produces E312 | ✓ PASS |

---

### TC-TRS-TRACE-006 — Verify that E313 is emitted for incompatible domain/reqDomain in satisfies: links.

**Verifies:** REQ-TRS-TRACE-006  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| software element satisfying hardware requirement produces E313 | ✓ PASS |
| software element satisfying system requirement produces no E313 | ✓ PASS |

---

### TC-TRS-TRACE-007 — Verify that E315 is emitted for cross-domain supertype: or typedBy: links.

**Verifies:** REQ-TRS-TRACE-007  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| software PartDef with hardware supertype produces E315 | ✓ PASS |

---

### TC-TRS-TRACE-008 — Verify that E314 is emitted for a deployment package with no hardware Allocation.

**Verifies:** REQ-TRS-TRACE-008  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| deployment package with no hardware allocation produces E314 | ✓ PASS |
| deployment package with hardware allocation produces no E314 | ✓ PASS |

---

### TC-TRS-VAL-001 — Verify that each parse-time error rule is triggered by the corresponding malformed input.

**Verifies:** REQ-TRS-VAL-001  
**Result:** ✓ PASS (19 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E001 | ✓ PASS |
| trigger E002 | ✓ PASS |
| trigger E004 | ✓ PASS |
| trigger E005 | ✓ PASS |
| trigger E006 | ✓ PASS |
| trigger E007 | ✓ PASS |
| trigger E008 | ✓ PASS |
| trigger E009 | ✓ PASS |
| trigger E010 | ✓ PASS |
| trigger E011 | ✓ PASS |
| trigger E012 | ✓ PASS |
| trigger E013 | ✓ PASS |
| trigger E014 | ✓ PASS |
| trigger E015 | ✓ PASS |
| trigger E300 | ✓ PASS |
| trigger E301 | ✓ PASS |
| trigger E302 | ✓ PASS |
| trigger E303 | ✓ PASS |
| trigger E304 | ✓ PASS |

---

### TC-TRS-VAL-002 — Verify that each model-time error rule is triggered by its cross-element condition.

**Verifies:** REQ-TRS-VAL-002  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger E101 | ✓ PASS |
| trigger E102 | ✓ PASS |
| trigger E103 | ✓ PASS |
| trigger E104 | ✓ PASS |
| trigger E105 | ✓ PASS |
| trigger E310 | ✓ PASS |
| trigger E311 | ✓ PASS |
| trigger E312 | ✓ PASS |
| trigger E313 | ✓ PASS |
| trigger E314 | ✓ PASS |
| trigger E315 | ✓ PASS |

---

### TC-TRS-VAL-003 — Verify that each warning rule is triggered by its condition with Warning severity.

**Verifies:** REQ-TRS-VAL-003  
**Result:** ✓ PASS (11 passed, 0 failed)

| Scenario | Result |
|---|---|
| trigger W001 | ✓ PASS |
| trigger W002 | ✓ PASS |
| trigger W003 | ✓ PASS |
| trigger W004 | ✓ PASS |
| trigger W005 | ✓ PASS |
| trigger W006 | ✓ PASS |
| trigger W300 | ✓ PASS |
| trigger W301 | ✓ PASS |
| trigger W302 | ✓ PASS |
| trigger W303 | ✓ PASS |
| trigger W304 | ✓ PASS |

---

### TC-TRS-VAL-004 — Verify that integrity-level propagation errors E841-E843 and W808 are enforced.

**Verifies:** REQ-TRS-VAL-004  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| E842: derivedFrom element missing integrity level | ✓ PASS |
| E843: satisfies element missing integrity level | ✓ PASS |
| W808: integrity level lower than source without breakdownAdr | ✓ PASS |

---

### TC-TRS-VAL-005 — Verify that each finding includes the required fields: rule code, element reference, and description.

**Verifies:** REQ-TRS-VAL-005,REQ-TRS-VAL-006  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| findings include rule code, element reference, and description | ✓ PASS |
| parse-time error is attributed to source file | ✓ PASS |

---

### TC-TRS-VAL-006 — Verify that E-code findings are marked Error and W-code findings are marked Warning.

**Verifies:** REQ-TRS-VAL-007  
**Result:** ✓ PASS (5 passed, 0 failed)

| Scenario | Result |
|---|---|
| E-code findings are under Errors section | ✓ PASS |
| W-code findings are under Warnings section | ✓ PASS |

---

### TC-TRS-VAL-007 — Verify that Error and Warning severity are reported consistently in the output.

**Verifies:** REQ-TRS-VAL-007  
**Result:** ✓ PASS (4 passed, 0 failed)

| Scenario | Result |
|---|---|
| Errors section does not contain W-codes | ✓ PASS |
| Warnings section does not contain E-codes | ✓ PASS |

---

### TC-TRS-XREF-001 — Verify that absolute qualified names are resolved correctly from the model root.

**Verifies:** REQ-TRS-XREF-001  
**Result:** ✓ PASS (3 passed, 0 failed)

| Scenario | Result |
|---|---|
| absolute supertype reference resolves correctly | ✓ PASS |
| absolute reference to non-existent element produces an error | ✓ PASS |

---

### TC-TRS-XREF-002 — Verify that relative references are resolved outward from the current package.

**Verifies:** REQ-TRS-XREF-002  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| sibling reference resolves within the same package | ✓ PASS |

---

### TC-TRS-XREF-003 — Verify that an unresolved cross-reference produces an error but does not abort processing.

**Verifies:** REQ-TRS-XREF-003  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| dangling reference produces an error without crashing | ✓ PASS |

---

### TC-TRS-XREF-004 — Verify that circular supertype chains are detected and reported without crashing.

**Verifies:** REQ-TRS-XREF-004  
**Result:** ✓ PASS (1 passed, 0 failed)

| Scenario | Result |
|---|---|
| two-element supertype cycle is detected without crashing | ✓ PASS |

---

### TC-TRS-XREF-005 — Verify that verifies: and derivedFrom: references are resolved by stable id:.

**Verifies:** REQ-TRS-XREF-005  
**Result:** ✓ PASS (2 passed, 0 failed)

| Scenario | Result |
|---|---|
| verifies: resolves by stable id: regardless of file path | ✓ PASS |

---
