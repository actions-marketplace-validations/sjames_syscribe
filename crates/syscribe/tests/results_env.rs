//! TC-TRS-RESENV-001 / REQ-TRS-RESENV-001: results are kept per TestEnvironment, and a TestPlan judges the
//! environments it runs on.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn te(id: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: TestEnvironment\nname: {id}\nstatus: available\nenvironmentKind: bench\n{extra}---\n\nRig.\n")
}

fn plan(id: &str, status: &str, runs_on: &str) -> String {
    format!("---\ntype: TestPlan\nid: {id}\nname: {id}\nstatus: {status}\nscope: hil\n{runs_on}testCases: [TC-EV-001]\n---\n\nPlan\n")
}

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-resenv-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Env\n---\n\nRoot.\n");
    write(&root, "REQ-EV-001.md", "---\nid: REQ-EV-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nBody.\n");
    write(
        &root,
        "TC-EV-001.md",
        "---\nid: TC-EV-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: T\nverifies: [REQ-EV-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: One\n    Given x\n    Then y\n```\n",
    );
    write(&root, "TE-EV-BENCH.md", &te("TE-EV-BENCH", ""));
    write(&root, "TE-EV-HIL.md", &te("TE-EV-HIL", ""));
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().expect("spawn syscribe");
    (String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned(), out.status.code().unwrap_or(-1))
}

fn rec(env: Option<&str>, verdict: &str) -> String {
    let p = env.map(|e| format!(r#","provenance":{{"environment":"{e}"}}"#)).unwrap_or_default();
    format!(r#"{{"testCase":"TC-EV-001","scenario":"One","verdict":"{verdict}"{p}}}"#)
}

fn ingest(root: &Path, records: &[String], extra: &[&str]) -> (String, String, i32) {
    write(root, "r.json", &format!("[{}]", records.join(",")));
    let file = root.join("r.json");
    let mut args = vec!["ingest-results", "--format", "records"];
    args.extend_from_slice(extra);
    args.push(file.to_str().unwrap());
    run(root, &args)
}

fn plan_verdict(root: &Path, id: &str) -> String {
    let (out, _, _) = run(root, &["testplan", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    v["testPlans"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()["verdict"].as_str().unwrap().to_string()
}

fn trace(root: &Path) -> String {
    run(root, &["trace", "REQ-EV-001"]).0
}

fn validate_lines(root: &Path, code: &str) -> Vec<String> {
    run(root, &["validate"]).0.lines().filter(|l| l.contains(code)).map(String::from).collect()
}

// ── recording ────────────────────────────────────────────────────────────────

#[test]
fn a_pass_on_the_bench_and_a_fail_on_the_hil_rig_are_both_kept_and_shown_per_rig() {
    let root = new_model();
    assert_eq!(ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass")], &[]).2, 0);
    assert_eq!(ingest(&root, &[rec(Some("TE-EV-HIL"), "fail")], &[]).2, 0);
    let t = trace(&root);
    assert!(t.contains("TC-EV-001 [fail] @ TE-EV-HIL"), "the failing rig is named, the passing one is not:\n{t}");
    assert!(!t.contains("TE-EV-BENCH"), "{t}");
    let (failures, _, _) = run(&root, &["results", "failures"]);
    assert!(failures.contains("TC-EV-001 / One @ TE-EV-HIL  fail"), "{failures}");
    let (json, _, _) = run(&root, &["results", "failures", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["records"][0]["environment"], "TE-EV-HIL");
}

#[test]
fn the_environment_flag_names_the_rig_for_records_that_do_not() {
    let root = new_model();
    let (out, err, code) = ingest(&root, &[rec(None, "pass"), rec(Some("TE-EV-BENCH"), "pass")], &["--environment", "TE-EV-HIL"]);
    assert_eq!(code, 0, "{out}{err}");
    let sc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap();
    let mut envs: Vec<&str> = sc["records"].as_array().unwrap().iter().map(|r| r["provenance"]["environment"].as_str().unwrap()).collect();
    envs.sort();
    assert_eq!(envs, ["TE-EV-BENCH", "TE-EV-HIL"], "the flag fills the gap; a record's own rig is kept");
}

#[test]
fn an_unknown_environment_flag_is_rejected_and_writes_nothing() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &[rec(None, "pass")], &["--environment", "TE-NOPE"]);
    assert_ne!(code, 0);
    assert!(err.contains("TE-NOPE"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
}

#[test]
fn the_environment_flag_is_refused_for_the_other_formats() {
    let root = new_model();
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    let (_, err, code) = run(&root, &["ingest-results", "--environment", "TE-EV-HIL", root.join("j.xml").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("--environment"), "{err}");
}

#[test]
fn the_latest_result_of_a_rig_supersedes_its_earlier_one_and_leaves_the_other_rig() {
    let root = new_model();
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass"), rec(Some("TE-EV-HIL"), "fail")], &[]);
    ingest(&root, &[rec(Some("TE-EV-HIL"), "pass")], &[]);
    assert!(trace(&root).contains("TC-EV-001 [pass]"), "{}", trace(&root));
}

// ── plans ────────────────────────────────────────────────────────────────────

#[test]
fn a_plan_judges_only_the_rigs_it_runs_on() {
    let root = new_model();
    write(&root, "TP-EV-HIL.md", &plan("TP-EV-HIL", "draft", "runsOn: [TE-EV-HIL]\n"));
    write(&root, "TP-EV-BENCH.md", &plan("TP-EV-BENCH", "draft", "runsOn: [TE-EV-BENCH]\n"));
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass"), rec(Some("TE-EV-HIL"), "fail")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-HIL"), "fail", "the hil failure decides the hil plan");
    assert_eq!(plan_verdict(&root, "TP-EV-BENCH"), "pass", "the bench plan does not see the hil failure");
}

#[test]
fn a_plan_with_two_rigs_needs_a_passing_result_on_each() {
    let root = new_model();
    write(&root, "TP-EV-BOTH.md", &plan("TP-EV-BOTH", "draft", "runsOn: [TE-EV-BENCH, TE-EV-HIL]\n"));
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-BOTH"), "incomplete", "no result on the hil rig yet");
    ingest(&root, &[rec(Some("TE-EV-HIL"), "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-BOTH"), "pass");
}

#[test]
fn a_missing_rig_in_an_approved_plan_raises_w615_naming_it_and_a_failure_says_failed() {
    let root = new_model();
    write(&root, "TP-EV-BOTH.md", &plan("TP-EV-BOTH", "approved", "runsOn: [TE-EV-BENCH, TE-EV-HIL]\n"));
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass")], &[]);
    let w = validate_lines(&root, "W615");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("TC-EV-001 @ TE-EV-HIL (missing)"), "{w:?}");
    ingest(&root, &[rec(Some("TE-EV-HIL"), "fail")], &[]);
    let w = validate_lines(&root, "W615");
    assert!(w[0].contains("TC-EV-001 @ TE-EV-HIL (FAILED)"), "{w:?}");
    ingest(&root, &[rec(Some("TE-EV-HIL"), "pass")], &[]);
    assert!(validate_lines(&root, "W615").is_empty());
}

#[test]
fn a_draft_plan_is_not_reported_by_w615() {
    let root = new_model();
    write(&root, "TP-EV-BOTH.md", &plan("TP-EV-BOTH", "draft", "runsOn: [TE-EV-BENCH, TE-EV-HIL]\n"));
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass")], &[]);
    assert!(validate_lines(&root, "W615").is_empty());
}

#[test]
fn a_plan_without_runs_on_is_conservative_and_a_failure_on_any_rig_fails_it() {
    let root = new_model();
    write(&root, "TP-EV-ANY.md", &plan("TP-EV-ANY", "draft", ""));
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass"), rec(Some("TE-EV-HIL"), "fail")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-ANY"), "fail");
}

#[test]
fn a_result_with_no_rig_does_not_count_for_a_plan_that_names_its_rigs() {
    let root = new_model();
    write(&root, "TP-EV-HIL.md", &plan("TP-EV-HIL", "draft", "runsOn: [TE-EV-HIL]\n"));
    ingest(&root, &[rec(None, "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-HIL"), "incomplete", "an unnamed rig is not the hil rig");
    // ...but a plan that names none still sees it
    write(&root, "TP-EV-ANY.md", &plan("TP-EV-ANY", "draft", ""));
    assert_eq!(plan_verdict(&root, "TP-EV-ANY"), "pass");
}

#[test]
fn a_waiver_on_one_rig_leaves_the_plan_incomplete_not_passing() {
    let root = new_model();
    write(&root, "TP-EV-BOTH.md", &plan("TP-EV-BOTH", "draft", "runsOn: [TE-EV-BENCH, TE-EV-HIL]\n"));
    let waived = r#"{"testCase":"TC-EV-001","scenario":"One","verdict":"waived","rationale":"rig down","provenance":{"environment":"TE-EV-HIL"}}"#.to_string();
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "pass"), waived], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-BOTH"), "incomplete");
}

// ── rig health ───────────────────────────────────────────────────────────────

#[test]
fn a_result_from_a_retired_rig_is_not_a_pass_for_the_plan() {
    let root = new_model();
    write(&root, "TE-EV-OLD.md", &te("TE-EV-OLD", "").replace("status: available", "status: retired"));
    write(&root, "TP-EV-OLD.md", &plan("TP-EV-OLD", "draft", "runsOn: [TE-EV-OLD]\n"));
    ingest(&root, &[rec(Some("TE-EV-OLD"), "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-OLD"), "incomplete");
}

#[test]
fn a_result_from_a_rig_past_its_calibration_is_not_a_pass_and_w892_still_names_the_rig() {
    let root = new_model();
    write(&root, "TE-EV-CAL.md", &te("TE-EV-CAL", "calibrationStatus: expired\n"));
    write(&root, "TP-EV-CAL.md", &plan("TP-EV-CAL", "approved", "runsOn: [TE-EV-CAL]\n"));
    ingest(&root, &[rec(Some("TE-EV-CAL"), "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-CAL"), "incomplete");
    assert!(validate_lines(&root, "W892").iter().any(|l| l.contains("TE-EV-CAL")), "{:?}", validate_lines(&root, "W892"));
    assert!(validate_lines(&root, "W615").iter().any(|l| l.contains("TC-EV-001 @ TE-EV-CAL (missing)")), "{:?}", validate_lines(&root, "W615"));
}

#[test]
fn a_healthy_rig_beside_an_unhealthy_one_is_unaffected() {
    let root = new_model();
    write(&root, "TE-EV-OLD.md", &te("TE-EV-OLD", "").replace("status: available", "status: retired"));
    write(&root, "TP-EV-HIL.md", &plan("TP-EV-HIL", "draft", "runsOn: [TE-EV-HIL]\n"));
    ingest(&root, &[rec(Some("TE-EV-OLD"), "fail"), rec(Some("TE-EV-HIL"), "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-HIL"), "pass");
}

// ── test cases with their own runsOn, configurations ─────────────────────────

#[test]
fn a_testcase_with_its_own_runs_on_is_judged_on_those_rigs() {
    let root = new_model();
    let tc = std::fs::read_to_string(root.join("TC-EV-001.md")).unwrap().replace("verifies: [REQ-EV-001]\n", "verifies: [REQ-EV-001]\nrunsOn: [TE-EV-HIL]\n");
    write(&root, "TC-EV-001.md", &tc);
    ingest(&root, &[rec(Some("TE-EV-BENCH"), "fail"), rec(Some("TE-EV-HIL"), "pass")], &[]);
    assert!(trace(&root).contains("TC-EV-001 [pass]"), "the bench failure is not on its rigs:\n{}", trace(&root));
}

#[test]
fn the_configuration_is_chosen_first_and_the_rig_filters_inside_it() {
    let root = new_model();
    write(&root, "CONF-EV.md", "---\nid: CONF-EV\ntype: Configuration\nname: C\nfeatures: {}\n---\n\nC.\n");
    let tc = std::fs::read_to_string(root.join("TC-EV-001.md")).unwrap().replace("verifies: [REQ-EV-001]\n", "verifies: [REQ-EV-001]\nrunsOn: [TE-EV-HIL]\n");
    write(&root, "TC-EV-001.md", &tc);
    // globally the hil rig failed; the configuration's own run of that rig passed, and its bench run failed
    ingest(&root, &[rec(Some("TE-EV-HIL"), "fail")], &[]);
    ingest(&root, &[rec(Some("TE-EV-HIL"), "pass"), rec(Some("TE-EV-BENCH"), "fail")], &["--config", "CONF-EV"]);
    let (lensed, _, _) = run(&root, &["trace", "REQ-EV-001", "--config", "CONF-EV"]);
    assert!(lensed.contains("TC-EV-001 [pass]"), "the configuration's own hil result wins, and its bench failure is not on the test's rigs:\n{lensed}");
    let (plain, _, _) = run(&root, &["trace", "REQ-EV-001"]);
    assert!(plain.contains("TC-EV-001 [fail]"), "with no configuration chosen a failure on any variant is never hidden:\n{plain}");
}

#[test]
fn records_on_a_rig_do_not_change_a_model_that_never_names_one() {
    let root = new_model();
    write(&root, "TP-EV-ANY.md", &plan("TP-EV-ANY", "approved", ""));
    ingest(&root, &[rec(None, "pass")], &[]);
    assert_eq!(plan_verdict(&root, "TP-EV-ANY"), "pass");
    assert!(validate_lines(&root, "W615").is_empty());
    assert!(!trace(&root).contains(" @ "), "{}", trace(&root));
}

#[test]
fn a_large_number_of_rigs_and_tests_judges_quickly() {
    let root = new_model();
    let mut runs = Vec::new();
    for i in 0..30 {
        write(&root, &format!("TE-EV-R{i:02}.md"), &te(&format!("TE-EV-R{i:02}"), ""));
        runs.push(format!("TE-EV-R{i:02}"));
    }
    write(&root, "TP-EV-WIDE.md", &plan("TP-EV-WIDE", "approved", &format!("runsOn: [{}]\n", runs.join(", "))));
    let recs: Vec<String> = runs.iter().map(|e| rec(Some(e), "pass")).collect();
    ingest(&root, &recs, &[]);
    let t = std::time::Instant::now();
    assert_eq!(plan_verdict(&root, "TP-EV-WIDE"), "pass");
    assert!(validate_lines(&root, "W615").is_empty());
    assert!(t.elapsed().as_secs() < 30, "{:?}", t.elapsed());
}
