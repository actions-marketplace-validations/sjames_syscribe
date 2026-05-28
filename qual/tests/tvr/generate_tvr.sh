#!/usr/bin/env bash
# qual/tests/tvr/generate_tvr.sh — generate Tool Validation Report from results.ndjson

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS="$SCRIPT_DIR/results.ndjson"
OUTPUT="$SCRIPT_DIR/TVR.md"
SYSCRIBE_VERSION="${1:-unknown}"

[ -f "$RESULTS" ] || { echo "No results file at $RESULTS" >&2; exit 1; }

python3 - "$RESULTS" "$OUTPUT" "$SYSCRIBE_VERSION" << 'EOF'
import sys, json
from datetime import datetime, timezone

results_path, output_path, version = sys.argv[1], sys.argv[2], sys.argv[3]
date = datetime.now(timezone.utc).strftime("%Y-%m-%d")

results = []
with open(results_path) as f:
    for line in f:
        line = line.strip()
        if line:
            results.append(json.loads(line))

total = len(results)
passed = sum(1 for r in results if r.get("verdict") == "PASS")
failed = total - passed
overall = "**PASS**" if failed == 0 else "**FAIL**"

lines = []
lines.append("# Tool Validation Report\n")
lines.append(f"**Tool:** syscribe CLI validator  ")
lines.append(f"**Version:** {version}  ")
lines.append(f"**Standard:** ISO 26262:2018 Part 8 §11 (TCL2), IEC 61508:2010 Part 3 Annex D  ")
lines.append(f"**Date:** {date}  ")
lines.append(f"**TRS:** `qual/Requirements/`  **Test cases:** `qual/TestCases/`\n")
lines.append("---\n")
lines.append("## 1. Summary\n")
lines.append("| Metric | Value |")
lines.append("|---|---|")
lines.append(f"| Total test cases | {total} |")
lines.append(f"| Passed | {passed} |")
lines.append(f"| Failed | {failed} |")
lines.append(f"| Overall verdict | {overall} |\n")
lines.append("---\n")
lines.append("## 2. Results\n")

for r in results:
    tc_id = r.get("tc_id", "")
    title = r.get("title", "")
    verifies = r.get("verifies", "")
    verdict = r.get("verdict", "FAIL")
    p = r.get("pass", 0)
    f = r.get("fail", 0)
    badge = "✓ PASS" if verdict == "PASS" else "✗ FAIL"

    lines.append(f"### {tc_id} — {title}\n")
    lines.append(f"**Verifies:** {verifies}  ")
    lines.append(f"**Result:** {badge} ({p} passed, {f} failed)\n")

    scenarios = r.get("scenarios", [])
    if scenarios:
        lines.append("| Scenario | Result |")
        lines.append("|---|---|")
        for s in scenarios:
            sv = "✓ PASS" if s.get("verdict") == "pass" else "✗ FAIL"
            lines.append(f"| {s.get('name','')} | {sv} |")
        lines.append("")

    lines.append("---\n")

with open(output_path, "w") as f:
    f.write("\n".join(lines))

print(f"TVR generated: {output_path} ({total} TCs, {passed} passed, {failed} failed)")
EOF
