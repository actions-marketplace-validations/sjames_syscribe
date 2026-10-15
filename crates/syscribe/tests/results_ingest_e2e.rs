//! End to end: every way of getting results into one model, side by side (REQ-TRS-RESREC-001, RESJOIN, RESENV,
//! RESPLUG, RESROBOT, RESCUKE). The per-format tests prove each in isolation; this proves they coexist.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn robot() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/robot/robot7.xml").to_str().unwrap().to_string()
}
fn cucumber() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cucumber/cucumber-js.json").to_str().unwrap().to_string()
}

fn tc(id: &str, ext: &str, extra_fm: &str, scenarios: &[&str]) -> String {
    let body: String = scenarios.iter().map(|s| format!("  Scenario: {s}\n    Given x\n    Then y\n\n")).collect();
    format!("---\nid: {id}\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"{id}\"\nverifies: [REQ-BRK-001]\n{ext}{extra_fm}---\n\n```gherkin\nFeature: f\n{body}```\n")
}

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-e2e-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    let w = |rel: &str, c: &str| std::fs::write(root.join(rel), c).unwrap();
    w("_index.md", "---\ntype: Package\nname: E2e\n---\n\nRoot.\n");
    w("REQ-BRK-001.md", "---\nid: REQ-BRK-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nThe brake shall respond.\n");
    // TC-BRK-001/002/003 are the ones the Robot and Cucumber fixtures tag; TC-JU has a unit test; TC-MAN is manual
    w("TC-BRK-001.md", &tc("TC-BRK-001", "", "", &["Request reaches the actuator in time", "Request during a bus fault is not lost"]));
    w("TC-BRK-002.md", &tc("TC-BRK-002", "", "", &["S"]));
    w("TC-BRK-003.md", &tc("TC-BRK-003", "extRef: \"TAP:parking-brake\"\n", "", &["P"]));
    w("TC-JU-001.md", &tc("TC-JU-001", "", "testFunctions:\n  - function: \"brake::tests::t_ok\"\n    scenario: \"U\"\nsourceFile: repo:x.rs\n", &["U"]));
    w("TC-MAN-001.md", &tc("TC-MAN-001", "", "", &["Manual one"]));
    w("TC-PLG-001.md", &tc("TC-PLG-001", "extRef: \"RIG:PLG-1\"\n", "", &["Plugin one"]));
    for (id, kind) in [("TE-E2E-BENCH", "bench"), ("TE-E2E-HIL", "hil"), ("TE-E2E-FIELD", "vehicle")] {
        w(&format!("{id}.md"), &format!("---\nid: {id}\ntype: TestEnvironment\nname: {id}\nstatus: available\nenvironmentKind: {kind}\n---\n\nRig.\n"));
    }
    w(
        "TP-E2E-001.md",
        "---\ntype: TestPlan\nid: TP-E2E-001\nname: P\nstatus: approved\nscope: hil\nrunsOn: [TE-E2E-HIL, TE-E2E-FIELD]\ntestCases: [TC-BRK-001, TC-BRK-003]\n---\n\nPlan\n",
    );
    w("junit.xml", "<testsuite><testcase classname=\"brake::tests\" name=\"t_ok\"/></testsuite>");
    w("session.json", r#"[{"testCase":"TC-MAN-001","scenario":"Manual one","steps":[{"cmd":"curl"}],"result":"pass"}]"#);
    w("bench.json", r#"[{"testCase":"TC-BRK-001","scenario":"Request reaches the actuator in time","verdict":"pass","provenance":{"environment":"TE-E2E-BENCH"}},{"testCase":"TC-BRK-001","scenario":"Request during a bus fault is not lost","verdict":"pass","provenance":{"environment":"TE-E2E-BENCH"}}]"#);
    w("report.txt", "rig log\n");
    let script = root.join("rig.sh");
    std::fs::write(&script, "#!/bin/sh\ncat >/dev/null\nprintf '%s' '{\"records\":[{\"testCase\":\"RIG:PLG-1\",\"scenario\":\"Plugin one\",\"verdict\":\"pass\"}],\"diagnostics\":[\"one case\"]}'\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    w(".syscribe.toml", "[plugins.rig]\ncommand = \"./rig.sh\"\nkind = \"results\"\ntimeout_ms = 5000\n");
    root
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().expect("spawn syscribe");
    (String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned(), out.status.code().unwrap_or(-1))
}

/// One ingest, by name.
fn ingest(root: &Path, which: &str) {
    let p = |f: &str| root.join(f).to_str().unwrap().to_string();
    let args: Vec<String> = match which {
        "junit" => vec!["ingest-results".into(), "--format".into(), "junit".into(), p("junit.xml")],
        "session" => vec!["ingest-results".into(), "--format".into(), "session-log".into(), p("session.json")],
        "records" => vec!["ingest-results".into(), "--format".into(), "records".into(), p("bench.json")],
        "robot" => vec!["ingest-results".into(), "--format".into(), "robot".into(), "--environment".into(), "TE-E2E-HIL".into(), robot()],
        "cucumber" => vec!["ingest-results".into(), "--format".into(), "cucumber".into(), "--environment".into(), "TE-E2E-FIELD".into(), cucumber()],
        "plugin" => vec!["ingest-results".into(), "--format".into(), "plugin:rig".into(), p("report.txt")],
        other => panic!("{other}"),
    };
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (out, err, code) = run(root, &refs);
    assert_eq!(code, 0, "{which}: {out}{err}");
}

const ALL: [&str; 6] = ["junit", "session", "records", "robot", "cucumber", "plugin"];

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

fn trace_lines(root: &Path) -> Vec<String> {
    run(root, &["trace", "REQ-BRK-001"]).0.lines().filter(|l| l.starts_with("| TC-")).map(|l| l.split('|').nth(1).unwrap().trim().to_string()).collect()
}

fn plan(root: &Path) -> String {
    let v: serde_json::Value = serde_json::from_str(&run(root, &["testplan", "--json"]).0).unwrap();
    v["testPlans"][0]["verdict"].as_str().unwrap().to_string()
}

#[test]
fn every_kind_of_result_lands_in_the_sidecar_and_none_displaces_another() {
    let root = new_model();
    for w in ALL {
        ingest(&root, w);
    }
    let sc = sidecar(&root);
    assert!(sc["by_leaf"].as_object().unwrap().contains_key("t_ok"), "junit: {sc}");
    assert!(sc["by_scenario"].as_object().unwrap().keys().any(|k| k.starts_with("TC-MAN-001::")), "session-log");
    let recs = sc["records"].as_array().unwrap();
    let from = |env: &str| recs.iter().filter(|r| r["provenance"]["environment"] == env).count();
    assert!(from("TE-E2E-BENCH") >= 2 && from("TE-E2E-HIL") >= 3 && from("TE-E2E-FIELD") >= 3, "records of all three rigs coexist: bench {} hil {} field {}", from("TE-E2E-BENCH"), from("TE-E2E-HIL"), from("TE-E2E-FIELD"));
    assert!(recs.iter().any(|r| r["testCase"] == "TC-PLG-001"), "the plugin's record, joined through its extRef");
    let t = trace_lines(&root);
    for needle in ["TC-JU-001 [pass]", "TC-MAN-001 [pass]", "TC-PLG-001 [pass]"] {
        assert!(t.iter().any(|l| l.contains(needle)), "{needle} in {t:?}");
    }
}

#[test]
fn the_order_of_the_ingests_does_not_change_what_the_readers_say() {
    let mut seen: Vec<(Vec<String>, String)> = Vec::new();
    let orders: [[&str; 6]; 3] = [ALL, ["plugin", "cucumber", "robot", "records", "session", "junit"], ["robot", "junit", "plugin", "records", "cucumber", "session"]];
    for order in orders {
        let root = new_model();
        for w in order {
            ingest(&root, w);
        }
        seen.push((trace_lines(&root), plan(&root)));
    }
    assert!(seen[0].0.len() >= 6, "the comparison is not vacuous: {:?}", seen[0]);
    assert!(seen[0].0.iter().any(|l| l.contains("[fail]")) && seen[0].0.iter().any(|l| l.contains("[pass]")), "{:?}", seen[0]);
    assert_eq!(seen[0], seen[1], "forwards and backwards");
    assert_eq!(seen[0], seen[2], "and shuffled");
}

#[test]
fn ingesting_everything_twice_changes_nothing() {
    let root = new_model();
    for w in ALL {
        ingest(&root, w);
    }
    let strip = |mut v: serde_json::Value| {
        // the ingest times differ by construction
        fn walk(v: &mut serde_json::Value) {
            match v {
                serde_json::Value::Object(m) => {
                    m.remove("ingested_at_unix");
                    m.values_mut().for_each(walk);
                }
                serde_json::Value::Array(a) => a.iter_mut().for_each(walk),
                _ => {}
            }
        }
        walk(&mut v);
        v
    };
    let first = strip(sidecar(&root));
    for w in ALL {
        ingest(&root, w);
    }
    let mut second = strip(sidecar(&root));
    let mut first = first;
    for v in [&mut first, &mut second] {
        if let Some(r) = v["records"].as_array_mut() {
            r.sort_by_key(|x| x.to_string());
        }
    }
    assert_eq!(first["records"], second["records"]);
    assert_eq!(first["by_leaf"], second["by_leaf"]);
    assert_eq!(first["by_scenario"], second["by_scenario"]);
}

#[test]
fn a_plan_over_two_rigs_is_judged_on_each_and_names_what_is_wrong() {
    let root = new_model();
    for w in ALL {
        ingest(&root, w);
    }
    // robot (hil) and cucumber (field) both fail TC-BRK-001's second scenario, so the plan fails
    assert_eq!(plan(&root), "fail");
    let w615: Vec<String> = run(&root, &["validate"]).0.lines().filter(|l| l.contains("W615")).map(String::from).collect();
    assert_eq!(w615.len(), 1, "{w615:?}");
    assert!(w615[0].contains("TC-BRK-001 @ TE-E2E-HIL (FAILED)") && w615[0].contains("TC-BRK-001 @ TE-E2E-FIELD (FAILED)"), "{w615:?}");
    // the bench passed it: a plan on the bench alone would pass
    let tp = std::fs::read_to_string(root.join("TP-E2E-001.md")).unwrap().replace("runsOn: [TE-E2E-HIL, TE-E2E-FIELD]", "runsOn: [TE-E2E-BENCH]").replace("testCases: [TC-BRK-001, TC-BRK-003]", "testCases: [TC-BRK-001]");
    std::fs::write(root.join("TP-E2E-001.md"), tp).unwrap();
    assert_eq!(plan(&root), "pass");
}

#[test]
fn fixing_a_result_on_one_rig_leaves_the_others_and_clears_only_its_part() {
    let root = new_model();
    for w in ALL {
        ingest(&root, w);
    }
    std::fs::write(
        root.join("hil-fixed.json"),
        r#"[{"testCase":"TC-BRK-001","scenario":"Request during a bus fault is not lost","verdict":"pass","provenance":{"environment":"TE-E2E-HIL"}},
            {"testCase":"TC-BRK-003","verdict":"pass","provenance":{"environment":"TE-E2E-FIELD"}}]"#,
    )
    .unwrap();
    let (_, err, code) = run(&root, &["ingest-results", "--format", "records", root.join("hil-fixed.json").to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let w615: Vec<String> = run(&root, &["validate"]).0.lines().filter(|l| l.contains("W615")).map(String::from).collect();
    assert_eq!(w615.len(), 1, "{w615:?}");
    assert!(!w615[0].contains("@ TE-E2E-HIL"), "the hil rig is fixed: {w615:?}");
    assert!(w615[0].contains("TC-BRK-001 @ TE-E2E-FIELD (FAILED)"), "the field rig still fails it: {w615:?}");
}

#[test]
fn editing_a_test_after_its_results_makes_every_kind_of_result_for_it_stale() {
    let root = new_model();
    for w in ALL {
        ingest(&root, w);
    }
    std::fs::write(root.join("TC-PLG-001.md"), tc("TC-PLG-001", "extRef: \"RIG:PLG-1\"\n", "", &["Plugin one"]) + "\nChanged.\n").unwrap();
    std::fs::write(root.join("TC-BRK-002.md"), tc("TC-BRK-002", "", "", &["S"]) + "\nChanged.\n").unwrap();
    let w091: Vec<String> = run(&root, &["validate"]).0.lines().filter(|l| l.contains("W091")).map(String::from).collect();
    assert!(w091.iter().any(|l| l.contains("TC-PLG-001")), "a plugin's result: {w091:?}");
    assert!(w091.iter().any(|l| l.contains("TC-BRK-002")), "a Robot or Cucumber result: {w091:?}");
    assert!(!w091.iter().any(|l| l.contains("TC-BRK-001.md")), "an untouched test is not stale: {w091:?}");
}

#[test]
fn history_diff_and_as_of_work_across_formats() {
    let root = new_model();
    let p = |f: &str| root.join(f).to_str().unwrap().to_string();
    let (_, e, c) = run(&root, &["ingest-results", "--format", "records", "--run", "good", &p("bench.json")]);
    assert_eq!(c, 0, "{e}");
    std::fs::write(root.join("bad.json"), r#"[{"testCase":"TC-BRK-001","scenario":"Request reaches the actuator in time","verdict":"fail","provenance":{"environment":"TE-E2E-BENCH"}}]"#).unwrap();
    let (_, e, c) = run(&root, &["ingest-results", "--format", "records", "--run", "bad", &p("bad.json")]);
    assert_eq!(c, 0, "{e}");
    let (out, _, code) = run(&root, &["results", "diff", "good", "bad", "--fail-on-regression"]);
    assert_eq!(code, 1, "{out}");
    let now = trace_lines(&root);
    let then = {
        let (o, _, _) = run(&root, &["--results-as-of", "good", "trace", "REQ-BRK-001"]);
        o.lines().filter(|l| l.starts_with("| TC-")).map(|l| l.split('|').nth(1).unwrap().trim().to_string()).collect::<Vec<_>>()
    };
    assert!(now.iter().any(|l| l.contains("TC-BRK-001 [fail]")), "{now:?}");
    assert!(then.iter().all(|l| !l.contains("TC-BRK-001 [fail]")), "as of the good run it did not fail: {then:?}");
}

#[test]
fn the_machine_readable_outputs_stay_valid_with_every_kind_of_result_present() {
    let root = new_model();
    for w in ALL {
        ingest(&root, w);
    }
    for cmd in [vec!["testplan", "--json"], vec!["matrix", "--json"], vec!["audit", "--json"], vec!["results", "failures", "--json"], vec!["results", "runs", "--json"]] {
        let (out, err, code) = run(&root, &cmd);
        assert!((0..=2).contains(&code), "{cmd:?}: {code} {err}");
        if !out.trim().is_empty() {
            serde_json::from_str::<serde_json::Value>(&out).unwrap_or_else(|e| panic!("{cmd:?} printed invalid JSON: {e}\n{out}"));
        }
    }
    let (out, _, _) = run(&root, &["validate", "--json"]);
    serde_json::from_str::<serde_json::Value>(&out).expect("validate --json");
}

#[test]
fn unmatched_results_of_every_format_are_listed_and_none_counts_as_evidence() {
    let root = new_model();
    for w in ["robot", "cucumber"] {
        ingest(&root, w);
    }
    let sc = sidecar(&root);
    // the latest records ingest owns `unmatched`: the Cucumber one
    assert!(sc["unmatched"].as_array().unwrap().iter().all(|r| r["testCase"].as_str().unwrap().starts_with("cucumber:")), "{sc}");
    let (failures, _, _) = run(&root, &["results", "failures"]);
    assert!(failures.contains("Unmatched records") && failures.contains("cucumber:"), "{failures}");
    assert!(sc["records"].as_array().unwrap().iter().all(|r| r["testCase"].as_str().unwrap().starts_with("TC-")), "an unmatched result is never a stored record");
}
