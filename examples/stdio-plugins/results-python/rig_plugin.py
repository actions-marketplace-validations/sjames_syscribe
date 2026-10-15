#!/usr/bin/env python3
"""A results plugin for a made-up rig log (REQ-TRS-RESPLUG-001, docs/model-guide/stdio-plugins.md).

Protocol: Syscribe writes one JSON request on stdin and reads one JSON object from stdout.

  request : {"protocolVersion": 1, "alias": ..., "reportPath": ..., "modelRoot": ...,
             "run"?: ..., "config"?: ..., "environment"?: ...}
  response: {"records": [...], "diagnostics": [...]}

Everything it returns goes through the same validator as `ingest-results --format records`.
Standard library only; no build step.
"""
import json
import re
import sys


def main():
    request = json.load(sys.stdin)
    records, diagnostics = [], []
    header = {}
    with open(request["reportPath"], encoding="utf-8") as f:
        for number, line in enumerate(f, 1):
            line = line.rstrip("\n")
            if line.startswith("RIGLOG"):
                header = dict(kv.split("=", 1) for kv in line.split()[2:] if "=" in kv)
                continue
            if not line.startswith("CASE "):
                continue
            parts = [p.strip() for p in line[5:].split("|")]
            if len(parts) < 3:
                diagnostics.append(f"line {number}: expected 'CASE <id> | <scenario> | <RESULT> | <details>', skipped")
                continue
            test, scenario, result = parts[0], parts[1], parts[2].upper()
            details = parts[3] if len(parts) > 3 else ""
            verdict = {"PASS": "pass", "FAIL": "fail", "BLOCKED": "blocked", "SKIP": "not-run"}.get(result)
            if verdict is None:
                diagnostics.append(f"line {number}: unknown result '{result}', skipped")
                continue
            record = {"testCase": test, "scenario": scenario, "verdict": verdict}
            m = re.search(r"latency=([\d.]+ ?ms)\s+limit<([\d.]+ ?ms)", details)
            if m:
                record["measurements"] = [{"name": "latency", "value": m.group(1), "limit": {"op": "<", "value": m.group(2)}}]
            provenance = {}
            if header.get("build"):
                provenance["build"] = {"commit": header["build"]}
            if header.get("operator"):
                provenance["operator"] = header["operator"]
            if request.get("environment"):
                provenance["environment"] = request["environment"]
            if provenance:
                record["provenance"] = provenance
            records.append(record)
    diagnostics.append(f"converted {len(records)} case(s) from {request['reportPath'].rsplit('/', 1)[-1]}")
    json.dump({"records": records, "diagnostics": diagnostics}, sys.stdout)


if __name__ == "__main__":
    main()
