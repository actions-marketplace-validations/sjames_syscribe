//! TC-TRS-RESCUKE-001 / REQ-TRS-RESCUKE-001: Cucumber JSON as a built-in results format.
//!
//! `cucumber-js.json` is real output of Cucumber.js 11.3.0 and `cucumber-ruby.json` of Cucumber-Ruby 3.1.2 (both from
//! `brake.feature`, beside them). `cucumber-jvm.json` is written by hand in the shape the JVM formatter documents
//! (separate backgrounds, hook arrays, `start_timestamp`, nanosecond durations, embeddings, a row of attempts per
//! retry): Cucumber-JVM could not be run here, and Cucumber.js never writes more than the last attempt of a retry.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cucumber").join(name).to_str().unwrap().to_string()
}

fn tc(id: &str, ext: &str, scenarios: &[&str]) -> String {
    let body: String = scenarios
        .iter()
        .map(|s| if s.starts_with("Outline:") { format!("  Scenario Outline: {}\n    Given x\n    Then y\n\n    Examples:\n      | a |\n      | 1 |\n\n", &s[8..].trim()) } else { format!("  Scenario: {s}\n    Given x\n    Then y\n\n") })
        .collect();
    format!("---\nid: {id}\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"{id}\"\nverifies: [REQ-BRK-001]\n{ext}---\n\n```gherkin\nFeature: f\n{body}```\n")
}

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-rescuke-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Ck\n---\n\nRoot.\n");
    write(&root, "REQ-BRK-001.md", "---\nid: REQ-BRK-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nBody.\n");
    write(&root, "TC-BRK-001.md", &tc("TC-BRK-001", "", &["Request reaches the actuator in time", "Request during a bus fault is not lost"]));
    write(&root, "TC-BRK-002.md", &tc("TC-BRK-002", "", &["S"]));
    write(&root, "TC-BRK-003.md", &tc("TC-BRK-003", "", &["P"]));
    write(&root, "TC-BRK-004.md", &tc("TC-BRK-004", "extRef: \"TAP:parking-brake\"\n", &["Parking brake holds"]));
    write(&root, "TC-OUT-001.md", &tc("TC-OUT-001", "", &["Outline: Latency by speed"]));
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
    let mut args = vec!["ingest-results", "--format", "cucumber"];
    args.extend_from_slice(extra);
    args.push(file);
    run(root, &args)
}

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

fn find<'a>(sc: &'a serde_json::Value, tc: &str, scenario: Option<&str>) -> &'a serde_json::Value {
    sc["records"].as_array().unwrap().iter().find(|r| r["testCase"] == tc && r["scenario"].as_str() == scenario).unwrap_or_else(|| panic!("no record {tc} / {scenario:?}: {sc}"))
}

/// The entry for the scenario called `name` among a record's `steps`.
fn entry<'a>(rec: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    rec["steps"].as_array().unwrap().iter().find(|s| s["scenario"] == name).unwrap_or_else(|| panic!("no entry {name} in {rec}"))
}

// ── minimal reports for focused cases ────────────────────────────────────────

fn st(kw: &str, name: &str, status: &str) -> serde_json::Value {
    let mut r = serde_json::json!({"status": status});
    if status != "skipped" && status != "undefined" {
        r["duration"] = 1000.into();
    }
    if status == "failed" {
        r["error_message"] = "Error: it broke\n    at somewhere (file.js:1:1)".into();
    }
    serde_json::json!({"keyword": kw, "name": name, "line": 1, "result": r})
}

fn scenario(id: &str, name: &str, line: u64, tags: &[&str], steps: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({"id": id, "keyword": "Scenario", "name": name, "line": line, "type": "scenario",
        "tags": tags.iter().map(|t| serde_json::json!({"name": t})).collect::<Vec<_>>(), "steps": steps})
}

fn feature(ftags: &[&str], elements: Vec<serde_json::Value>) -> String {
    serde_json::to_string(&serde_json::json!([{"uri": "f.feature", "id": "f", "keyword": "Feature", "name": "F",
        "tags": ftags.iter().map(|t| serde_json::json!({"name": t})).collect::<Vec<_>>(), "elements": elements}]))
    .unwrap()
}

fn ingest_json(root: &Path, json: &str, extra: &[&str]) -> (String, String, i32) {
    write(root, "r.json", json);
    ingest(root, root.join("r.json").to_str().unwrap(), extra)
}

// ── the three implementations ────────────────────────────────────────────────

#[test]
fn real_cucumber_js_and_ruby_and_the_jvm_shaped_report_are_read_alike() {
    for (file, fail_msg) in [("cucumber-js.json", "message dropped after fault"), ("cucumber-ruby.json", "message dropped after fault"), ("cucumber-jvm.json", "message dropped after fault")] {
        let root = new_model();
        let (out, err, code) = ingest(&root, &fixture(file), &[]);
        assert_eq!(code, 0, "{file}: {out}{err}");
        let sc = sidecar(&root);
        let ok = find(&sc, "TC-BRK-001", Some("Request reaches the actuator in time"));
        assert_eq!(ok["verdict"], "pass", "{file}");
        let bad = find(&sc, "TC-BRK-001", Some("Request during a bus fault is not lost"));
        assert_eq!(bad["verdict"], "fail", "{file}");
        let e = entry(bad, "Request during a bus fault is not lost");
        assert_eq!(e["step"], "Then the actuator still receives it", "{file}");
        assert!(e["message"].as_str().unwrap().contains(fail_msg), "{file}: {e}");
        assert!(!e["message"].as_str().unwrap().contains("\n") && !e["message"].as_str().unwrap().contains("    at "), "{file}: only the first line of an error, not its stack trace: {e}");
    }
}

#[test]
fn a_failing_hook_fails_its_scenario_in_every_implementation() {
    for file in ["cucumber-js.json", "cucumber-ruby.json", "cucumber-jvm.json"] {
        let root = new_model();
        assert_eq!(ingest(&root, &fixture(file), &[]).2, 0, "{file}");
        let sc = sidecar(&root);
        let rec = if file == "cucumber-jvm.json" { find(&sc, "TC-BRK-001", None) } else { find(&sc, "TC-BRK-002", None) };
        let e = entry(rec, "Hook failure");
        assert_eq!(e["status"], "failed", "{file}: {e}");
        assert!(e["step"].as_str().unwrap().contains("hook"), "{file}: {e}");
        assert!(e["message"].as_str().unwrap().contains("rig not reachable"), "{file}: {e}");
    }
}

#[test]
fn pending_undefined_and_stepless_scenarios_are_not_run_and_an_ambiguous_one_fails() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("cucumber-jvm.json"), &[]).2, 0);
    let sc = sidecar(&root);
    let two = find(&sc, "TC-BRK-002", None);
    for name in ["Pending", "Undefined", "No steps"] {
        assert_eq!(entry(two, name)["status"], "not run", "{name}");
    }
    let one = find(&sc, "TC-BRK-001", None);
    assert_eq!(entry(one, "Ambiguous")["status"], "failed", "an ambiguous step definition is a failure");
}

#[test]
fn js_and_ruby_agree_on_pending_and_undefined() {
    for file in ["cucumber-js.json", "cucumber-ruby.json"] {
        let root = new_model();
        assert_eq!(ingest(&root, &fixture(file), &[]).2, 0);
        let rec = find(&sidecar(&root), "TC-BRK-002", None).clone();
        assert_eq!(entry(&rec, "A pending one")["status"], "not run", "{file}");
        assert_eq!(entry(&rec, "An undefined one")["status"], "not run", "{file}");
    }
}

// ── background ───────────────────────────────────────────────────────────────

#[test]
fn a_failing_background_step_fails_the_scenarios_after_it() {
    let root = new_model();
    let bg = serde_json::json!({"type": "background", "keyword": "Background", "name": "", "line": 3, "steps": [st("Given ", "the rig is up", "failed")]});
    let s1 = scenario("f;a", "Request reaches the actuator in time", 8, &["@TC-BRK-001"], vec![st("When ", "x", "skipped")]);
    let (_, err, code) = ingest_json(&root, &feature(&[], vec![bg, s1]), &[]);
    assert_eq!(code, 0, "{err}");
    let rec = find(&sidecar(&root), "TC-BRK-001", Some("Request reaches the actuator in time")).clone();
    assert_eq!(rec["verdict"], "fail");
    assert_eq!(entry(&rec, "Request reaches the actuator in time")["step"], "Given the rig is up");
}

#[test]
fn a_background_that_passes_does_not_hide_a_failing_scenario_and_the_next_background_replaces_it() {
    let root = new_model();
    let passing = serde_json::json!({"type": "background", "keyword": "Background", "name": "", "line": 3, "steps": [st("Given ", "ok", "passed")]});
    let failing = serde_json::json!({"type": "background", "keyword": "Background", "name": "", "line": 3, "steps": [st("Given ", "bad", "failed")]});
    let els = vec![
        passing.clone(),
        scenario("f;a", "Request reaches the actuator in time", 8, &["@TC-BRK-001"], vec![st("When ", "w", "passed")]),
        failing,
        scenario("f;b", "Request during a bus fault is not lost", 13, &["@TC-BRK-001"], vec![st("When ", "w", "skipped")]),
        passing,
        scenario("f;c", "Another", 20, &["@TC-BRK-002"], vec![st("When ", "w", "passed")]),
    ];
    assert_eq!(ingest_json(&root, &feature(&[], els), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-001", Some("Request reaches the actuator in time"))["verdict"], "pass");
    assert_eq!(find(&sc, "TC-BRK-001", Some("Request during a bus fault is not lost"))["verdict"], "fail");
    assert_eq!(find(&sc, "TC-BRK-002", None)["verdict"], "pass", "the later passing background applies to the later scenario");
}

// ── outlines ─────────────────────────────────────────────────────────────────

#[test]
fn outline_rows_are_numbered_and_the_outline_title_carries_the_worst_row() {
    for file in ["cucumber-js.json", "cucumber-ruby.json", "cucumber-jvm.json"] {
        let root = new_model();
        // the fixtures tag their outline TC-BRK-003; point a model TestCase with the outline's title at it
        write(&root, "TC-BRK-003.md", &tc("TC-BRK-003", "", &["Outline: Latency by speed"]));
        assert_eq!(ingest(&root, &fixture(file), &[]).2, 0, "{file}");
        let sc = sidecar(&root);
        for (n, v) in [(1, "pass"), (2, "pass"), (3, "fail")] {
            assert_eq!(find(&sc, "TC-BRK-003", Some(&format!("Latency by speed [row {n}]")))["verdict"], v, "{file} row {n}");
        }
        let outline = find(&sc, "TC-BRK-003", Some("Latency by speed"));
        assert_eq!(outline["verdict"], "fail", "{file}: the outline's own title carries the worst of its rows");
        assert_eq!(outline["steps"][0]["scenario"], "Latency by speed [row 3]", "{file}: the failing row leads");
    }
}

#[test]
fn rows_with_the_same_id_are_told_apart_by_their_line_as_cucumber_js_writes_them() {
    let root = new_model();
    let row = |line: u64, status: &str| {
        let mut e = scenario("f;latency", "Latency", line, &["@TC-BRK-002"], vec![st("Then ", "x", status)]);
        e["keyword"] = "Scenario Outline".into();
        e
    };
    assert_eq!(ingest_json(&root, &feature(&[], vec![row(10, "passed"), row(11, "failed"), row(12, "passed")]), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-002", Some("Latency [row 2]"))["verdict"], "fail", "three rows, not one scenario with three attempts");
    assert_eq!(find(&sc, "TC-BRK-002", Some("Latency [row 1]"))["verdict"], "pass");
}

// ── retries ──────────────────────────────────────────────────────────────────

#[test]
fn a_pass_after_a_failure_is_flaky_a_failure_on_the_last_attempt_is_a_failure_and_two_passes_are_a_pass() {
    let root = new_model();
    let els = vec![
        scenario("f;flaky", "Flaky", 5, &["@TC-BRK-001"], vec![st("When ", "w", "failed")]),
        scenario("f;flaky", "Flaky", 5, &["@TC-BRK-001"], vec![st("When ", "w", "passed")]),
        scenario("f;always", "Always", 9, &["@TC-BRK-002"], vec![st("When ", "w", "passed")]),
        scenario("f;always", "Always", 9, &["@TC-BRK-002"], vec![st("When ", "w", "failed")]),
        scenario("f;steady", "Steady", 14, &["@TC-BRK-003"], vec![st("When ", "w", "passed")]),
        scenario("f;steady", "Steady", 14, &["@TC-BRK-003"], vec![st("When ", "w", "passed")]),
    ];
    assert_eq!(ingest_json(&root, &feature(&[], els), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-001", None)["verdict"], "flaky");
    assert_eq!(entry(find(&sc, "TC-BRK-001", None), "Flaky")["message"], "Error: it broke", "the failed attempt is what the reader needs to see");
    assert_eq!(find(&sc, "TC-BRK-002", None)["verdict"], "fail");
    assert_eq!(find(&sc, "TC-BRK-003", None)["verdict"], "pass");
}

#[test]
fn the_jvm_style_retry_attempts_are_folded_in_the_real_shaped_report() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("cucumber-jvm.json"), &[]).2, 0);
    let two = find(&sidecar(&root), "TC-BRK-002", None).clone();
    assert_eq!(entry(&two, "Flaky one")["status"], "flaky");
    assert_eq!(entry(&two, "Always fails")["status"], "failed");
    assert_eq!(entry(&two, "Always fails")["message"], "java.lang.RuntimeException: attempt 3", "the last attempt's failure");
    assert_eq!(two["steps"].as_array().unwrap().iter().filter(|s| s["scenario"] == "Always fails").count(), 1, "three attempts are one scenario");
}

#[test]
fn flaky_is_not_a_pass_for_the_readers() {
    let root = new_model();
    let els = vec![
        scenario("f;flaky", "Flaky", 5, &["@TC-BRK-002"], vec![st("When ", "w", "failed")]),
        scenario("f;flaky", "Flaky", 5, &["@TC-BRK-002"], vec![st("When ", "w", "passed")]),
    ];
    ingest_json(&root, &feature(&[], els), &[]);
    let t = run(&root, &["trace", "REQ-BRK-001"]).0;
    assert!(t.contains("TC-BRK-002 [unknown]"), "a flaky test proves nothing:\n{t}");
}

// ── joining ──────────────────────────────────────────────────────────────────

#[test]
fn tags_join_by_id_by_extref_in_any_case_and_from_the_feature() {
    let root = new_model();
    let els = vec![
        scenario("f;a", "Request reaches the actuator in time", 5, &["@TC-BRK-001"], vec![st("When ", "w", "passed")]),
        scenario("f;b", "Parking brake holds", 9, &["@extref:TAP:parking-brake"], vec![st("When ", "w", "passed")]),
        scenario("f;c", "Upper", 12, &["@EXTREF:TAP:parking-brake"], vec![st("When ", "w", "failed")]),
        scenario("f;d", "From the feature", 15, &[], vec![st("When ", "w", "passed")]),
    ];
    assert_eq!(ingest_json(&root, &feature(&["@TC-BRK-002"], els), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-001", Some("Request reaches the actuator in time"))["verdict"], "pass");
    assert_eq!(find(&sc, "TC-BRK-004", Some("Parking brake holds"))["verdict"], "pass", "joined through @extref:, and the title matched a Scenario:");
    assert_eq!(find(&sc, "TC-BRK-004", None)["verdict"], "fail", "@EXTREF: in any case joins too; its title matches no Scenario:, so it is the TestCase's own verdict");
    assert_eq!(find(&sc, "TC-BRK-002", None)["steps"].as_array().unwrap().len(), 4, "a feature-level tag applies to every scenario");
}

#[test]
fn requirement_tags_and_other_tags_are_ignored_and_untagged_scenarios_are_reported() {
    let root = new_model();
    let els = vec![
        scenario("f;a", "A", 5, &["@REQ-BRK-001", "@smoke", "@TC-BRK-001"], vec![st("When ", "w", "passed")]),
        scenario("f;b", "B", 9, &["@REQ-BRK-001", "@smoke"], vec![st("When ", "w", "passed")]),
    ];
    let (out, _, code) = ingest_json(&root, &feature(&[], els), &[]);
    assert_eq!(code, 0);
    assert!(out.contains("1 of 2 record(s) name no TestCase") && out.contains("cucumber:F.B"), "{out}");
    assert_eq!(sidecar(&root)["unmatched"].as_array().unwrap().len(), 1);
    let els = vec![scenario("f;b", "B", 9, &["@smoke"], vec![st("When ", "w", "passed")])];
    let (_, err, code) = ingest_json(&root, &feature(&[], els), &["--strict"]);
    assert_eq!(code, 1);
    assert!(err.contains("cucumber:F.B"), "{err}");
}

#[test]
fn several_scenarios_for_one_testcase_fold_with_the_worst_verdict_and_the_failure_first() {
    let root = new_model();
    let els = vec![
        scenario("f;a", "First", 5, &["@TC-BRK-002"], vec![st("When ", "w", "passed")]),
        scenario("f;b", "Second", 9, &["@TC-BRK-002"], vec![st("When ", "w", "failed")]),
        scenario("f;c", "Third", 12, &["@TC-BRK-002"], vec![st("When ", "w", "skipped")]),
    ];
    assert_eq!(ingest_json(&root, &feature(&[], els), &[]).2, 0);
    let rec = find(&sidecar(&root), "TC-BRK-002", None).clone();
    assert_eq!(rec["verdict"], "fail");
    assert_eq!(rec["steps"][0]["scenario"], "Second", "the failing scenario leads");
    assert_eq!(rec["steps"].as_array().unwrap().len(), 3);
}

#[test]
fn a_scenario_with_two_matching_tags_records_for_both_testcases() {
    let root = new_model();
    let els = vec![scenario("f;a", "A", 5, &["@TC-BRK-002", "@TC-BRK-003", "@TC-BRK-002"], vec![st("When ", "w", "passed")])];
    assert_eq!(ingest_json(&root, &feature(&[], els), &[]).2, 0);
    let sc = sidecar(&root);
    assert_eq!(find(&sc, "TC-BRK-002", None)["steps"].as_array().unwrap().len(), 1);
    assert_eq!(find(&sc, "TC-BRK-003", None)["steps"].as_array().unwrap().len(), 1);
}

// ── provenance and what is not stored ────────────────────────────────────────

#[test]
fn start_timestamps_and_durations_become_the_span_and_command_line_values_fill_the_rest() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &fixture("cucumber-jvm.json"), &["--environment", "TE-BRK-001", "--build-commit", "abc1234", "--operator", "alice"]);
    assert_eq!(code, 0, "{err}");
    let p = find(&sidecar(&root), "TC-BRK-001", Some("Request reaches the actuator in time"))["provenance"].clone();
    assert_eq!(p["startedAt"], "2026-10-01T08:00:00.000Z");
    assert_eq!(p["finishedAt"], "2026-10-01T08:00:00.007Z", "the start plus the background (1.5 ms), both steps (2 and 3 ms) and the after hook (0.3 ms): 6.8 ms, in the same zone");
    assert_eq!((p["environment"].as_str(), p["build"]["commit"].as_str(), p["operator"].as_str()), (Some("TE-BRK-001"), Some("abc1234"), Some("alice")));
}

#[test]
fn a_report_with_no_timestamps_gets_none_invented() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("cucumber-js.json"), &[]).2, 0);
    let rec = find(&sidecar(&root), "TC-BRK-001", Some("Request reaches the actuator in time")).clone();
    assert!(rec["provenance"].get("startedAt").is_none() || rec["provenance"]["startedAt"].is_null(), "{rec}");
}

#[test]
fn embeddings_and_output_are_never_stored() {
    let root = new_model();
    assert_eq!(ingest(&root, &fixture("cucumber-jvm.json"), &[]).2, 0);
    let text = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    assert!(!text.contains("log line from the test"), "step output is not stored");
    assert!(!text.contains("eHh4eHh4"), "embedded data (base64) is not stored");
    assert!(text.len() < 40_000, "the sidecar stays small: {} bytes", text.len());
}

#[test]
fn an_artefact_is_referenced_by_hash_and_utc_offset_is_refused() {
    let root = new_model();
    write(&root, "report.html", "abc");
    let (_, err, code) = ingest(&root, &fixture("cucumber-js.json"), &["--artefact", root.join("report.html").to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(find(&sidecar(&root), "TC-BRK-001", Some("Request reaches the actuator in time"))["provenance"]["artefacts"][0]["hash"], "blake3:6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85");
    let root = new_model();
    let (_, err, code) = ingest(&root, &fixture("cucumber-js.json"), &["--utc-offset", "+01:00"]);
    assert_ne!(code, 0);
    assert!(err.contains("--utc-offset"), "{err}");
}

// ── files that are not Cucumber json ─────────────────────────────────────────

#[test]
fn the_message_protocol_other_json_and_broken_input_are_clean_errors_that_write_nothing() {
    let ndjson = std::fs::read_to_string(fixture("messages.ndjson")).unwrap();
    let bad_status = feature(&[], vec![scenario("f;a", "A", 1, &["@TC-BRK-001"], vec![st("When ", "w", "maybe")])]);
    let no_result = r#"[{"uri":"f","name":"F","elements":[{"type":"scenario","name":"A","keyword":"Scenario","tags":[{"name":"@TC-BRK-001"}],"steps":[{"keyword":"When ","name":"w"}]}]}]"#.to_string();
    let cases: Vec<(&str, String, &str)> = vec![
        ("ndjson", ndjson, "NDJSON message protocol"),
        ("one message", r#"{"meta":{"protocolVersion":"27.2.0"}}"#.to_string(), "message protocol"),
        ("object", r#"{"hello":"world"}"#.to_string(), "found an object"),
        ("string", r#""x""#.to_string(), "found a string"),
        ("empty array", "[]".to_string(), "no features"),
        ("no scenarios", r#"[{"uri":"f","name":"F","elements":[]}]"#.to_string(), "no scenarios"),
        ("unknown status", bad_status, "'maybe'"),
        ("no result", no_result, "no result"),
        ("truncated", std::fs::read_to_string(fixture("cucumber-js.json")).unwrap()[..500].to_string(), "not valid JSON"),
        ("xml", "<testsuite/>".to_string(), "not valid JSON"),
        ("feature not an object", "[1]".to_string(), "must be an object"),
    ];
    for (name, body, needle) in cases {
        let root = new_model();
        let (_, err, code) = ingest_json(&root, &body, &[]);
        assert_ne!(code, 0, "{name}");
        assert!(err.contains(needle), "{name} should mention '{needle}': {err}");
        assert!(!err.contains("panicked"), "{name}: {err}");
        assert!(!root.join(".syscribe/results.json").exists(), "{name}: nothing written");
    }
}

#[test]
fn a_byte_order_mark_is_tolerated_and_a_missing_file_is_an_error() {
    let root = new_model();
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(std::fs::read_to_string(fixture("cucumber-js.json")).unwrap().as_bytes());
    std::fs::write(root.join("bom.json"), bytes).unwrap();
    assert_eq!(ingest(&root, root.join("bom.json").to_str().unwrap(), &[]).2, 0);
    let (_, err, code) = ingest(&root, "/nonexistent/cucumber.json", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("cucumber.json"), "{err}");
}

// ── detection ────────────────────────────────────────────────────────────────

#[test]
fn a_cucumber_json_file_is_recognised_without_a_format_and_cargo_json_still_is_not_mistaken() {
    let root = new_model();
    let (out, err, code) = run(&root, &["ingest-results", &fixture("cucumber-ruby.json")]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("(cucumber)"), "{out}");
    write(&root, "cargo.json", "{\"type\":\"test\",\"name\":\"a::b\",\"event\":\"ok\"}\n");
    let (out, _, code) = run(&root, &["ingest-results", root.join("cargo.json").to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(out.contains("(cargo-json)"), "{out}");
}

// ── the rest of the pipeline ─────────────────────────────────────────────────

#[test]
fn run_config_replace_a_second_rig_and_staleness_work() {
    let root = new_model();
    write(&root, "CONF-BRK.md", "---\nid: CONF-BRK\ntype: Configuration\nname: C\nfeatures: {}\n---\n\nC.\n");
    assert_eq!(ingest(&root, &fixture("cucumber-js.json"), &["--run", "r1", "--environment", "TE-BRK-001"]).2, 0);
    assert_eq!(ingest(&root, &fixture("cucumber-ruby.json"), &["--config", "CONF-BRK"]).2, 0);
    assert!(run(&root, &["results", "runs"]).0.contains("r1"));
    assert_eq!(ingest(&root, &fixture("cucumber-js.json"), &["--replace"]).2, 0);
    write(&root, "TC-BRK-001.md", &(tc("TC-BRK-001", "", &["Request reaches the actuator in time", "Request during a bus fault is not lost"]) + "\nA new paragraph.\n"));
    let (out, _, _) = run(&root, &["validate"]);
    assert!(out.lines().any(|l| l.contains("W091") && l.contains("TC-BRK-001")), "{out}");
}

#[test]
fn the_verdicts_reach_trace() {
    let root = new_model();
    ingest(&root, &fixture("cucumber-js.json"), &["--environment", "TE-BRK-001"]);
    let t = run(&root, &["trace", "REQ-BRK-001"]).0;
    assert!(t.contains("TC-BRK-001 [fail] @ TE-BRK-001"), "{t}");
}

#[test]
fn a_large_report_is_read_quickly() {
    let root = new_model();
    let els: Vec<_> = (0..5000).map(|i| scenario(&format!("f;s{i}"), &format!("S{i}"), i + 1, &[if i % 500 == 0 { "@TC-BRK-002" } else { "@other" }], vec![st("When ", "w", if i % 97 == 0 { "failed" } else { "passed" })])).collect();
    let t = std::time::Instant::now();
    let (out, err, code) = ingest_json(&root, &feature(&[], els), &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(t.elapsed().as_secs() < 30, "{:?}", t.elapsed());
    assert_eq!(find(&sidecar(&root), "TC-BRK-002", None)["steps"].as_array().unwrap().len(), 10);
}

#[test]
fn the_worked_example_behaves_as_its_readme_says() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/results-cucumber");
    let tmp = std::env::temp_dir().join(format!("syscribe-cuke-example-{}", std::process::id()));
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
    for file in ["cucumber-js.json", "cucumber-ruby.json", "cucumber-jvm.json"] {
        let (out, err, code) = run(&model, &["ingest-results", tmp.join(file).to_str().unwrap(), "--environment", "TE-HIL-001", "--build-commit", "abc1234"]);
        assert_eq!(code, 0, "{file}: {out}{err}");
        assert!(out.contains("(cucumber)"), "{file}: recognised by shape: {out}");
        let t = run(&model, &["trace", "REQ-BRK-001"]).0;
        assert!(t.contains("TC-BRK-001 [fail] @ TE-HIL-001"), "{file}: {t}");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
