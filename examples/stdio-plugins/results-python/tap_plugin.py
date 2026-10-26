#!/usr/bin/env python3
"""A results plugin for TAP 13 output (REQ-TRS-RESPLUG-001).

The first word of a test description is the test's key (a TestCase id or an `extRef`), the rest is the scenario
title when it matches one. `# SKIP` becomes `not-run`, `# TODO` is ignored. Standard library only.
"""
import json
import re
import sys

LINE = re.compile(r"^(not ok|ok)\s+\d+\s*-?\s*(\S+)\s*(.*?)\s*(#.*)?$")


def main():
    request = json.load(sys.stdin)
    records, diagnostics = [], []
    with open(request["reportPath"], encoding="utf-8") as f:
        for number, line in enumerate(f, 1):
            m = LINE.match(line.strip())
            if not m:
                continue
            status, key, title, directive = m.group(1), m.group(2), m.group(3), (m.group(4) or "").upper()
            if "TODO" in directive:
                continue
            verdict = "not-run" if "SKIP" in directive else ("pass" if status == "ok" else "fail")
            record = {"testCase": key, "verdict": verdict}
            if title:
                record["scenario"] = title
            if request.get("environment"):
                record["provenance"] = {"environment": request["environment"]}
            records.append(record)
    json.dump({"records": records, "diagnostics": diagnostics}, sys.stdout)


if __name__ == "__main__":
    main()
