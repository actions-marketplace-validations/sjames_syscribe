//! TC-TRS-RESROBOT-001 / REQ-TRS-RESROBOT-001: Robot Framework `output.xml` as a built-in results format.
//!
//! The fixtures under `tests/fixtures/robot/` are real files written by Robot Framework 7.4.2 and 6.1.1.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/robot").join(name).to_str().unwrap().to_string()
}

fn tc(id: &str, ext: &str, scenarios: &[&str]) -> String {
    let body: String = scenarios.iter().map(|s| format!("  Scenario: {s}\n    Given x\n    Then y\n\n")).collect();
    format!("---\nid: {id}\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"{id}\"\nverifies: [REQ-BRK-001]\n{ext}---\n\n```gherkin\nFeature: f\n{body}```\n")
}

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-resrobot-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Rb\n---\n\nRoot.\n");
    write(&root, "REQ-BRK-001.md", "---\nid: REQ-BRK-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nBody.\n");
    write(&root, "TC-BRK-001.md", &tc("TC-BRK-001", "", &["Request reaches the actuator in time", "Request during a bus fault is not lost"]));
    write(&root, "TC-BRK-002.md", &tc("TC-BRK-002", "", &["S"]));
    write(&root, "TC-BRK-003.md", &tc("TC-BRK-003", "extRef: \"TAP:parking-brake\"\n", &["P"]));
    write(&root, "TE-BRK-001.md", "---\nid: TE-BRK-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: hil\n---\n\nRig.\n");
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

fn ingest(root: &Path, file: &str, extra: &[&str]) -> (String, String, i32) {
    let mut args = vec!["ingest-results", "--format", "robot"];
    args.extend_from_slice(extra);
    args.push(file);
    run(root, &args)
}

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

fn find<'a>(sc: &'a serde_json::Value, tc: &str, scenario: Option<&str>) -> &'a serde_json::Value {
    sc["records"].as_array().unwrap().iter().find(|r| r["testCase"] == tc && r["scenario"].as_str() == scenario).unwrap_or_else(|| panic!("no record for {tc} / {scenario:?} in {sc}"))
}

fn trace(root: &Path) -> String {
    run(root, &["trace", "REQ-BRK-001"]).0
}

// ── both Robot versions ──────────────────────────────────────────────────────

#[test]
fn robot_7_and_robot_6_files_are_read_alike() {
    for (file, tool, start_prefix) in [("robot7.xml", "Robot 7.4.2", "2026-10-15T01:43:24"), ("robot6.xml", "Robot 6.1.1", "2026-10-15T01:43:41")] {
        let root = new_model();
        let (out, err, code) = ingest(&root, &fixture(file), &[]);
        assert_eq!(code, 0, "{file}: {out}{err}");
        let sc = sidecar(&root);
        let timed = find(&sc, "TC-BRK-001", Some("Request reaches the actuator in time"));
        assert_eq!(timed["verdict"], "pass", "{file}");
        assert_eq!(timed["provenance"]["tool"], tool);
        assert!(timed["provenance"]["startedAt"].as_str().unwrap().starts_with(start_prefix), "{file}: {timed}");
        assert!(timed["provenance"]["startedAt"].as_str().unwrap().ends_with('Z'));
        assert!(timed["provenance"]["finishedAt"].as_str().unwrap() >= timed["provenance"]["startedAt"].as_str().unwrap(), "{file}: finish is not before start");
        let fault = find(&sc, "TC-BRK-001", Some("Request during a bus fault is not lost"));
        assert_eq!(fault["verdict"], "fail");
        assert_eq!(fault["steps"][0]["message"], "message dropped after fault", "{file}");
        assert_eq!(find(&sc, "TC-BRK-002", None)["verdict"], "not-run", "SKIP is not-run");
        assert_eq!(find(&sc, "TC-BRK-003", None)["verdict"], "pass", "joined through extref:TAP:parking-brake");
    }
}

#[test]
fn tests_with_no_tag_that_names_a_testcase_are_counted_reported_and_kept_apart() {
    let root = new_model();
    let (out, _, code) = ingest(&root, &fixture("robot7.xml"), &[]);
    assert_eq!(code, 0);
    assert!(out.contains("2 of 6 record(s) name no TestCase"), "{out}");
    assert!(out.contains("Untagged test") && out.contains("Not a testcase tag"), "{out}");
    let sc = sidecar(&root);
    assert_eq!(sc["unmatched"].as_array().unwrap().len(), 2);
    assert!(sc["records"].as_array().unwrap().iter().all(|r| r["testCase"].as_str().unwrap().starts_with("TC-")), "no unmatched test is evidence");
    let (_, err, code) = ingest(&root, &fixture("robot7.xml"), &["--strict"]);
    assert_eq!(code, 1);
    assert!(err.contains("Untagged test"), "{err}");
}

#[test]
fn requirement_and_other_tags_are_ignored_and_never_an_error() {
    // `Not a testcase tag` carries REQ-BRK-001 (a Requirement id) and `smoke`: neither is a TestCase, neither fails the file
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("robot7.xml"), &[]).2, 0);
}

#[test]
fn the_verdict_and_the_detail_reach_the_readers() {
    let root = new_model();
    ingest(&root, &fixture("robot7.xml"), &[]);
    let t = trace(&root);
    assert!(t.contains("TC-BRK-001 [fail]") && t.contains("TC-BRK-003 [pass]") && t.contains("TC-BRK-002 [unknown]"), "{t}");
}

// ── how a Robot status becomes a verdict ─────────────────────────────────────

#[test]
fn a_test_that_never_ran_because_a_parent_setup_failed_is_blocked_not_failed() {
    for file in ["setupfail7.xml", "setupfail6.xml"] {
        let root = new_model();
        assert_eq!(ingest(&root, &fixture(file), &[]).2, 0, "{file}");
        let sc = sidecar(&root);
        for tc in ["TC-BRK-001", "TC-BRK-002"] {
            let r = find(&sc, tc, None);
            assert_eq!(r["verdict"], "blocked", "{file} {tc}");
            assert!(r["steps"][0]["message"].as_str().unwrap().contains("rig not ready"), "{r}");
        }
    }
}

#[test]
fn a_test_whose_own_setup_body_or_teardown_failed_is_a_failure() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("own7.xml"), &[]).2, 0);
    let sc = sidecar(&root);
    // TC-BRK-001 has two tests: one whose own setup failed, one templated (second row fails): both fail
    assert_eq!(find(&sc, "TC-BRK-001", None)["verdict"], "fail");
    assert_eq!(find(&sc, "TC-BRK-002", None)["verdict"], "fail", "the body passed, the teardown failed: the test failed");
}

// ── nested suites, special characters ────────────────────────────────────────

#[test]
fn nested_suites_give_full_names_and_xml_entities_are_decoded() {
    for file in ["nested7.xml", "nested6.xml"] {
        let root = new_model();
        assert_eq!(ingest(&root, &fixture(file), &[]).2, 0, "{file}");
        let sc = sidecar(&root);
        let r = find(&sc, "TC-BRK-001", None);
        let step = &r["steps"][0];
        assert_eq!(step["test"], "Nested.Outer.Inner.Deep.Deep test <&> \"quoted\"", "{file}: {step}");
        let msg = step["message"].as_str().unwrap();
        assert!(msg.contains("line one") && msg.contains("line two & <tag> \"q\""), "{file}: entities decoded, multi-line folded: {msg}");
        assert!(!msg.contains('\n'), "one line");
        assert_eq!(find(&sc, "TC-BRK-002", None)["verdict"], "pass", "a test in the outer suite");
    }
}

// ── several tests, one TestCase ──────────────────────────────────────────────

fn two_tests(first: &str, second: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<robot generator="Robot 7.0 (x)" rpa="false" schemaversion="5"><suite id="s1" name="S">
<test id="s1-t1" name="First"><tag>TC-BRK-002</tag><status status="{first}" start="2026-01-01T10:00:00.000000" elapsed="1.5">{}</status></test>
<test id="s1-t2" name="Second"><tag>TC-BRK-002</tag><status status="{second}" start="2026-01-01T10:00:05.000000" elapsed="2.0">{}</status></test>
<status status="PASS" start="2026-01-01T10:00:00.000000" elapsed="8"/></suite></robot>"#,
        if first == "FAIL" { "first broke" } else { "" },
        if second == "FAIL" { "second broke" } else { "" },
    )
}

#[test]
fn several_tests_for_one_testcase_fold_with_the_worst_verdict_and_the_failure_first() {
    let root = new_model();
    write(&root, "two.xml", &two_tests("PASS", "FAIL"));
    assert_eq!(ingest(&root, root.join("two.xml").to_str().unwrap(), &[]).2, 0);
    let sc = sidecar(&root);
    let recs: Vec<_> = sc["records"].as_array().unwrap().iter().filter(|r| r["testCase"] == "TC-BRK-002").collect();
    assert_eq!(recs.len(), 1, "one record for one TestCase");
    assert_eq!(recs[0]["verdict"], "fail");
    assert_eq!(recs[0]["steps"][0]["test"], "S.Second", "the failing test leads");
    assert_eq!(recs[0]["steps"][0]["message"], "second broke");
    assert_eq!(recs[0]["steps"].as_array().unwrap().len(), 2);
    assert_eq!(recs[0]["provenance"]["startedAt"], "2026-01-01T10:00:00.000000Z", "kept as Robot wrote it, plus the zone");
    assert_eq!(recs[0]["provenance"]["finishedAt"], "2026-01-01T10:00:07.000Z", "the span of both tests");
}

#[test]
fn a_pass_with_a_skip_is_not_a_pass_and_two_passes_are() {
    let root = new_model();
    write(&root, "a.xml", &two_tests("PASS", "SKIP"));
    ingest(&root, root.join("a.xml").to_str().unwrap(), &[]);
    assert_eq!(find(&sidecar(&root), "TC-BRK-002", None)["verdict"], "not-run");
    let root = new_model();
    write(&root, "b.xml", &two_tests("PASS", "PASS"));
    ingest(&root, root.join("b.xml").to_str().unwrap(), &[]);
    assert_eq!(find(&sidecar(&root), "TC-BRK-002", None)["verdict"], "pass");
}

#[test]
fn a_test_with_two_matching_tags_records_for_both_testcases_once_each() {
    let root = new_model();
    let xml = r#"<robot generator="Robot 7.0" rpa="false"><suite name="S"><test name="Both"><tag>TC-BRK-002</tag><tag>TC-BRK-003</tag><tag>TC-BRK-002</tag><status status="PASS" start="2026-01-01T10:00:00.000000" elapsed="1"/></test></suite></robot>"#;
    write(&root, "x.xml", xml);
    assert_eq!(ingest(&root, root.join("x.xml").to_str().unwrap(), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(sc["records"].as_array().unwrap().len(), 2);
    assert_eq!(find(&sc, "TC-BRK-002", None)["steps"].as_array().unwrap().len(), 1, "a repeated tag does not record twice");
}

#[test]
fn the_extref_prefix_is_case_insensitive_and_an_older_tags_wrapper_is_read() {
    let root = new_model();
    let xml = r#"<robot generator="Robot 4.0" rpa="false"><suite name="S"><test name="T"><tags><tag>EXTREF:TAP:parking-brake</tag></tags><status status="PASS" starttime="20260101 10:00:00.000" endtime="20260101 10:00:01.000"/></test></suite></robot>"#;
    write(&root, "x.xml", xml);
    assert_eq!(ingest(&root, root.join("x.xml").to_str().unwrap(), &[]).2, 0);
    assert_eq!(find(&sidecar(&root), "TC-BRK-003", None)["verdict"], "pass");
}

#[test]
fn a_test_named_like_a_scenario_is_recorded_against_it_and_others_against_the_testcase() {
    let root = new_model();
    let xml = r#"<robot generator="Robot 7.0" rpa="false"><suite name="S">
<test name="Request reaches the actuator in time"><tag>TC-BRK-001</tag><status status="PASS" start="2026-01-01T10:00:00.000000" elapsed="1"/></test>
<test name="Some other check"><tag>TC-BRK-001</tag><status status="PASS" start="2026-01-01T10:00:01.000000" elapsed="1"/></test>
</suite></robot>"#;
    write(&root, "x.xml", xml);
    assert_eq!(ingest(&root, root.join("x.xml").to_str().unwrap(), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-001", Some("Request reaches the actuator in time"))["verdict"], "pass");
    assert_eq!(find(&sc, "TC-BRK-001", None)["verdict"], "pass");
}

// ── files that are not Robot results ─────────────────────────────────────────

#[test]
fn other_xml_truncated_xml_rpa_and_empty_results_are_clean_errors_that_write_nothing() {
    let cases: Vec<(&str, String, &str)> = vec![
        ("junit", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>".into(), "<testsuite>"),
        ("rpa", "<robot rpa=\"true\"><suite name=\"s\"><task name=\"t\"/></suite></robot>".into(), "RPA"),
        ("empty", "<robot rpa=\"false\"><suite name=\"s\"/></robot>".into(), "no tests"),
        ("truncated", std::fs::read_to_string(fixture("robot7.xml")).unwrap()[..900].to_string(), "robot"),
        ("notxml", "this is not xml at all".into(), "robot"),
        ("unknownstatus", r#"<robot rpa="false"><suite name="s"><test name="t"><tag>TC-BRK-001</tag><status status="MAYBE" start="2026-01-01T10:00:00.000000" elapsed="1"/></test></suite></robot>"#.into(), "MAYBE"),
        ("nostatus", r#"<robot rpa="false"><suite name="s"><test name="t"><tag>TC-BRK-001</tag></test></suite></robot>"#.into(), "no <status>"),
    ];
    for (name, xml, needle) in cases {
        let root = new_model();
        write(&root, "x.xml", &xml);
        let (_, err, code) = ingest(&root, root.join("x.xml").to_str().unwrap(), &[]);
        assert_ne!(code, 0, "{name}");
        assert!(err.contains(needle), "{name}: {err}");
        assert!(!err.contains("panicked"), "{name}: {err}");
        assert!(!root.join(".syscribe/results.json").exists(), "{name}: nothing written");
    }
}

#[test]
fn a_missing_file_is_an_error() {
    let root = new_model();
    let (_, err, code) = ingest(&root, "/nonexistent/output.xml", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("output.xml"), "{err}");
}

// ── detection ────────────────────────────────────────────────────────────────

#[test]
fn an_xml_file_is_detected_by_its_root_element_when_no_format_is_given() {
    let root = new_model();
    let (out, err, code) = run(&root, &["ingest-results", &fixture("robot7.xml")]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("(robot)"), "a Robot file is read as Robot: {out}");
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    let (out, _, code) = run(&root, &["ingest-results", root.join("j.xml").to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(out.contains("(junit)"), "{out}");
}

// ── provenance given on the command line ─────────────────────────────────────

#[test]
fn environment_commit_and_operator_come_from_the_command_line() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &fixture("robot7.xml"), &["--environment", "TE-BRK-001", "--build-commit", "abc1234", "--operator", "alice"]);
    assert_eq!(code, 0, "{err}");
    let p = find(&sidecar(&root), "TC-BRK-001", Some("Request reaches the actuator in time"))["provenance"].clone();
    assert_eq!((p["environment"].as_str(), p["build"]["commit"].as_str(), p["operator"].as_str()), (Some("TE-BRK-001"), Some("abc1234"), Some("alice")));
    assert_eq!(p["tool"], "Robot 7.4.2");
}

#[test]
fn an_artefact_is_referenced_by_its_hash_and_never_copied() {
    let root = new_model();
    write(&root, "log.html", "abc");
    let log = root.join("log.html");
    let (_, err, code) = ingest(&root, &fixture("robot7.xml"), &["--artefact", log.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let a = find(&sidecar(&root), "TC-BRK-003", None)["provenance"]["artefacts"][0].clone();
    assert!(a["uri"].as_str().unwrap().starts_with("file://") && a["uri"].as_str().unwrap().ends_with("log.html"), "{a}");
    // BLAKE3("abc"), a published test vector: an independent check of the hash
    assert_eq!(a["hash"], "blake3:6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85");
    write(&root, "secret.txt", "TOP-SECRET-CONTENT");
    let secret = root.join("secret.txt");
    assert_eq!(ingest(&root, &fixture("robot7.xml"), &["--artefact", secret.to_str().unwrap()]).2, 0);
    let sidecar_text = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    assert!(!sidecar_text.contains("TOP-SECRET-CONTENT"), "the file's contents are never copied");
}

#[test]
fn a_missing_or_non_file_artefact_rejects_the_ingest() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &fixture("robot7.xml"), &["--artefact", "/nonexistent/log.html"]);
    assert_ne!(code, 0);
    assert!(err.contains("log.html"), "{err}");
    let (_, err, code) = ingest(&root, &fixture("robot7.xml"), &["--artefact", root.to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("not a file"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
}

#[test]
fn several_artefacts_are_all_recorded() {
    let root = new_model();
    write(&root, "a.html", "a");
    write(&root, "b.png", "b");
    let (a, b) = (root.join("a.html"), root.join("b.png"));
    assert_eq!(ingest(&root, &fixture("robot7.xml"), &["--artefact", a.to_str().unwrap(), "--artefact", b.to_str().unwrap()]).2, 0);
    assert_eq!(find(&sidecar(&root), "TC-BRK-003", None)["provenance"]["artefacts"].as_array().unwrap().len(), 2);
}

#[test]
fn robot_timestamps_have_no_zone_so_the_zone_is_the_users_to_state() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("robot6.xml"), &["--utc-offset", "+02:00"]).2, 0);
    let p = find(&sidecar(&root), "TC-BRK-001", Some("Request reaches the actuator in time"))["provenance"].clone();
    assert!(p["startedAt"].as_str().unwrap().ends_with("+02:00") && p["finishedAt"].as_str().unwrap().ends_with("+02:00"), "{p}");
    for bad in ["02:00", "+2:00", "+25:00", "UTC", "+02:60"] {
        let root = new_model();
        let (_, err, code) = ingest(&root, &fixture("robot6.xml"), &["--utc-offset", bad]);
        assert_ne!(code, 0, "{bad}");
        assert!(err.contains("--utc-offset"), "{bad}: {err}");
    }
    let root = new_model();
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    let (_, err, code) = run(&root, &["ingest-results", "--format", "junit", "--utc-offset", "Z", root.join("j.xml").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("--utc-offset"), "{err}");
}

#[test]
fn the_new_flags_are_refused_for_the_formats_that_do_not_take_them() {
    let root = new_model();
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    for flag in [["--operator", "bob"], ["--artefact", "j.xml"]] {
        let (_, err, code) = run(&root, &["ingest-results", "--format", "junit", flag[0], flag[1], root.join("j.xml").to_str().unwrap()]);
        assert_ne!(code, 0, "{flag:?}");
        assert!(err.contains(flag[0]), "{flag:?}: {err}");
    }
}

// ── the rest of the pipeline ─────────────────────────────────────────────────

#[test]
fn run_config_replace_and_a_second_rig_work_with_robot() {
    let root = new_model();
    write(&root, "CONF-BRK.md", "---\nid: CONF-BRK\ntype: Configuration\nname: C\nfeatures: {}\n---\n\nC.\n");
    assert_eq!(ingest(&root, &fixture("robot7.xml"), &["--run", "r1", "--environment", "TE-BRK-001"]).2, 0);
    assert_eq!(ingest(&root, &fixture("robot6.xml"), &["--config", "CONF-BRK"]).2, 0);
    assert!(run(&root, &["results", "runs"]).0.contains("r1"));
    assert!(sidecar(&root)["by_config"]["CONF-BRK"]["records"].as_array().is_some());
    assert_eq!(ingest(&root, &fixture("robot7.xml"), &["--replace"]).2, 0);
}

#[test]
fn staleness_applies_to_robot_results_too() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("robot7.xml"), &[]).2, 0);
    write(&root, "TC-BRK-003.md", &(tc("TC-BRK-003", "extRef: \"TAP:parking-brake\"\n", &["P"]) + "\nA new paragraph.\n"));
    let (out, _, _) = run(&root, &["validate"]);
    assert!(out.lines().any(|l| l.contains("W091") && l.contains("TC-BRK-003")), "{out}");
}

#[test]
fn a_large_result_file_is_read_quickly() {
    let root = new_model();
    let mut xml = String::from(r#"<robot generator="Robot 7.0" rpa="false"><suite name="Big">"#);
    for i in 0..5000 {
        let tag = if i % 500 == 0 { "TC-BRK-002" } else { "other" };
        xml.push_str(&format!(r#"<test name="Test {i}"><kw name="k"><status status="PASS" start="2026-01-01T10:00:00.000000" elapsed="0.1"/></kw><tag>{tag}</tag><status status="{}" start="2026-01-01T10:00:00.000000" elapsed="0.1"/></test>"#, if i % 97 == 0 { "FAIL" } else { "PASS" }));
    }
    xml.push_str("</suite></robot>");
    write(&root, "big.xml", &xml);
    let t = std::time::Instant::now();
    let (out, err, code) = ingest(&root, root.join("big.xml").to_str().unwrap(), &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(t.elapsed().as_secs() < 30, "{:?}", t.elapsed());
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-002", None)["steps"].as_array().unwrap().len(), 10, "ten tests carry the tag");
    assert!(sc["unmatched"].as_array().unwrap().len() >= 4000);
}

#[test]
fn a_fresh_run_of_the_installed_robot_ingests() {
    // Robot itself writes the file, so a newer format change would show here. Skipped when Robot is not installed.
    if Command::new("robot").arg("--version").output().is_err() {
        eprintln!("robot not installed; skipping");
        return;
    }
    let root = new_model();
    write(&root, "s.robot", "*** Test Cases ***\nA pass\n    [Tags]    TC-BRK-002\n    Log    fine\nA fail\n    [Tags]    TC-BRK-003\n    Fail    broke\n");
    let out = Command::new("robot").current_dir(&root).args(["--output", "live.xml", "--log", "NONE", "--report", "NONE", "s.robot"]).output().unwrap();
    assert!(root.join("live.xml").exists(), "{}", String::from_utf8_lossy(&out.stdout));
    let (_, err, code) = ingest(&root, root.join("live.xml").to_str().unwrap(), &[]);
    assert_eq!(code, 0, "{err}");
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-002", None)["verdict"], "pass");
    assert_eq!(find(&sc, "TC-BRK-003", None)["verdict"], "fail");
}

#[test]
fn the_worked_example_behaves_as_its_readme_says() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/results-robot");
    let tmp = std::env::temp_dir().join(format!("syscribe-robot-example-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    fn copy(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for e in std::fs::read_dir(from).unwrap() {
            let e = e.unwrap();
            if e.file_type().unwrap().is_dir() {
                copy(&e.path(), &to.join(e.file_name()));
            } else {
                std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
            }
        }
    }
    copy(&src, &tmp);
    let model = tmp.join("model");
    let (out, err, code) = run(&model, &["ingest-results", tmp.join("output.xml").to_str().unwrap(), "--environment", "TE-HIL-001", "--build-commit", "abc1234", "--operator", "alice"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("(robot)") && out.contains("2 of 6 record(s) name no TestCase"), "{out}");
    let t = run(&model, &["trace", "REQ-BRK-001"]).0;
    assert!(t.contains("TC-BRK-001 [fail] @ TE-HIL-001") && t.contains("TC-BRK-003 [pass]") && t.contains("TC-BRK-002 [unknown]"), "{t}");
    let _ = std::fs::remove_dir_all(&tmp);
}
