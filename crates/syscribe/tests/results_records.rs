//! TC-TRS-RESREC-001 / REQ-TRS-RESREC-001: `ingest-results --format records`, the normalised result record with
//! verdicts, measurements and provenance.
//!
//! Black-box: a minimal model in a fresh temp directory per test, driven through the `syscribe` binary.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-resrec-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Rec\n---\n\nModel root.\n");
    write(&root, "REQ-RR-001.md", "---\nid: REQ-RR-001\ntype: Requirement\nname: \"A requirement\"\nstatus: approved\nreqDomain: system\n---\n\nBody.\n");
    for (id, scenarios) in [("TC-RR-001", ["One", "Two"]), ("TC-RR-002", ["One", "Two"]), ("TC-RR-003", ["One", "Two"])] {
        let body: String = scenarios.iter().map(|s| format!("  Scenario: {s}\n    Given x\n    Then y\n\n")).collect();
        write(&root, &format!("{id}.md"), &format!("---\nid: {id}\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"{id}\"\nverifies: [REQ-RR-001]\n---\n\n```gherkin\nFeature: f\n{body}```\n"));
    }
    write(&root, "TE-RR-BENCH.md", "---\nid: TE-RR-BENCH\ntype: TestEnvironment\nname: Bench\nstatus: available\nenvironmentKind: bench\n---\n\nBench.\n");
    write(&root, "TE-RR-HIL.md", "---\nid: TE-RR-HIL\ntype: TestEnvironment\nname: Hil\nstatus: available\nenvironmentKind: hil\n---\n\nHil.\n");
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

fn ingest(root: &Path, name: &str, json: &str, extra: &[&str]) -> (String, String, i32) {
    write(root, name, json);
    let file = root.join(name);
    let mut args = vec!["ingest-results", "--format", "records"];
    args.extend_from_slice(extra);
    args.push(file.to_str().unwrap());
    run(root, &args)
}

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

const ALL_VERDICTS: &str = r#"[
  {"testCase": "TC-RR-001", "scenario": "One", "verdict": "pass"},
  {"testCase": "TC-RR-001", "scenario": "Two", "verdict": "waived", "rationale": "rig down, accepted by review RR-7"},
  {"testCase": "TC-RR-002", "scenario": "One", "verdict": "pass"},
  {"testCase": "TC-RR-002", "scenario": "Two", "verdict": "blocked"},
  {"testCase": "TC-RR-003", "scenario": "One", "verdict": "pass"},
  {"testCase": "TC-RR-003", "scenario": "Two", "verdict": "pass"}
]"#;

#[test]
fn every_verdict_is_ingested_and_waived_is_distinct_from_pass() {
    let root = new_model();
    let (out, err, code) = ingest(&root, "r.json", ALL_VERDICTS, &[]);
    assert_eq!(code, 0, "{out}{err}");
    let (trace, _, _) = run(&root, &["trace", "REQ-RR-001"]);
    assert!(trace.contains("TC-RR-001 [waived]"), "pass + waived is waived, not pass:\n{trace}");
    assert!(trace.contains("TC-RR-002 [unknown]"), "pass + blocked proves nothing, so it stays unknown:\n{trace}");
    assert!(trace.contains("TC-RR-003 [pass]"), "{trace}");
}

#[test]
fn a_failing_record_beats_a_waived_one() {
    let root = new_model();
    let (_, err, code) = ingest(
        &root,
        "r.json",
        r#"[{"testCase":"TC-RR-001","scenario":"One","verdict":"fail"},{"testCase":"TC-RR-001","scenario":"Two","verdict":"waived","rationale":"x"}]"#,
        &[],
    );
    assert_eq!(code, 0, "{err}");
    let (trace, _, _) = run(&root, &["trace", "REQ-RR-001"]);
    assert!(trace.contains("TC-RR-001 [fail]"), "{trace}");
}

#[test]
fn a_record_without_a_scenario_is_the_verdict_of_the_whole_testcase() {
    let root = new_model();
    let (_, err, code) = ingest(&root, "r.json", r#"[{"testCase":"TC-RR-001","verdict":"pass"},{"testCase":"TC-RR-002","verdict":"fail"}]"#, &[]);
    assert_eq!(code, 0, "{err}");
    let (trace, _, _) = run(&root, &["trace", "REQ-RR-001"]);
    assert!(trace.contains("TC-RR-001 [pass]") && trace.contains("TC-RR-002 [fail]"), "{trace}");
}

#[test]
fn malformed_records_fail_closed_and_name_the_record() {
    let cases: &[(&str, &str)] = &[
        (r#"[{"testCase":"TC-RR-001","verdict":"great"}]"#, "great"),
        (r#"[{"testCase":"TC-RR-001","verdict":"waived"}]"#, "rationale"),
        (r#"[{"testCase":"TC-RR-001","verdict":"waived","rationale":"  "}]"#, "rationale"),
        (r#"[{"testCase":"","verdict":"pass"}]"#, "testCase"),
        (r#"[{"testCase":"TC-RR-001","verdict":"pass","provenance":{"startedAt":"yesterday"}}]"#, "startedAt"),
        (r#"[{"testCase":"TC-RR-001","verdict":"pass","provenance":{"artefacts":[{"uri":"file:///x","hash":"md5:abc"}]}}]"#, "hash"),
        (r#"[{"testCase":"TC-RR-001","verdict":"pass","provenance":{"artefacts":[{"uri":"file:///x","hash":"sha256:abc"}]}}]"#, "hash"),
        (r#"[{"testCase":"TC-RR-001","verdict":"pass","provenance":{"environment":"TE-NOPE"}}]"#, "TE-NOPE"),
        (r#"{"testCase":"TC-RR-001","verdict":"pass"}"#, "array"),
        (r#"[]"#, "no records"),
    ];
    for (json, needle) in cases {
        let root = new_model();
        let (out, err, code) = ingest(&root, "r.json", json, &[]);
        assert_ne!(code, 0, "{json} was accepted: {out}");
        assert!(err.contains(needle), "{json}: stderr should mention {needle}: {err}");
        assert!(!root.join(".syscribe/results.json").exists(), "nothing written for {json}");
    }
}

#[test]
fn a_malformed_ingest_leaves_an_existing_sidecar_untouched() {
    let root = new_model();
    ingest(&root, "ok.json", ALL_VERDICTS, &[]);
    let before = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    let (_, _, code) = ingest(&root, "bad.json", r#"[{"testCase":"TC-RR-001","verdict":"nope"}]"#, &[]);
    assert_ne!(code, 0);
    assert_eq!(before, std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap());
}

#[test]
fn measurements_are_checked_against_their_limits_with_units() {
    let m = |verdict: &str, value: &str, op: &str, lim: &str| {
        format!(r#"[{{"testCase":"TC-RR-001","verdict":"{verdict}","measurements":[{{"name":"latency","value":"{value}","limit":{{"op":"{op}","value":"{lim}"}}}}]}}]"#)
    };
    // within limit, pass: accepted
    let root = new_model();
    assert_eq!(ingest(&root, "r.json", &m("pass", "4.2 ms", "<", "5 ms"), &[]).2, 0);
    // violated limit but verdict pass: rejected
    let root = new_model();
    let (_, err, code) = ingest(&root, "r.json", &m("pass", "6 ms", "<", "5 ms"), &[]);
    assert_ne!(code, 0);
    assert!(err.contains("latency"), "{err}");
    // violated limit with verdict fail: consistent, accepted
    let root = new_model();
    assert_eq!(ingest(&root, "r.json", &m("fail", "6 ms", "<", "5 ms"), &[]).2, 0);
    // different units of one dimension compare correctly: 4000 us < 5 ms
    let root = new_model();
    assert_eq!(ingest(&root, "r.json", &m("pass", "4000 us", "<", "5 ms"), &[]).2, 0);
    // limit in another dimension: rejected
    let root = new_model();
    let (_, err, code) = ingest(&root, "r.json", &m("pass", "4 ms", "<", "5 m"), &[]);
    assert_ne!(code, 0);
    assert!(err.contains("dimension"), "{err}");
    // unknown operator: rejected
    let root = new_model();
    assert_ne!(ingest(&root, "r.json", &m("pass", "4 ms", "~", "5 ms"), &[]).2, 0);
}

#[test]
fn provenance_is_kept_and_optional() {
    let root = new_model();
    let json = r#"[
      {"testCase":"TC-RR-001","scenario":"One","verdict":"pass","provenance":{
        "build":{"commit":"abc1234","version":"1.2.3"},"environment":"TE-RR-HIL","operator":"alice",
        "startedAt":"2026-10-01T08:00:00Z","finishedAt":"2026-10-01T08:00:05.250+02:00",
        "artefacts":[{"uri":"s3://rig/log1.txt","hash":"sha256:0000000000000000000000000000000000000000000000000000000000000000"}]}},
      {"testCase":"TC-RR-001","scenario":"Two","verdict":"pass"}]"#;
    let (out, err, code) = ingest(&root, "r.json", json, &[]);
    assert_eq!(code, 0, "{out}{err}");
    let sc = sidecar(&root);
    let recs = sc["records"].as_array().expect("records section");
    assert_eq!(recs.len(), 2);
    let first = recs.iter().find(|r| r["scenario"] == "One").unwrap();
    assert_eq!(first["provenance"]["environment"], "TE-RR-HIL", "stored by id");
    assert_eq!(first["provenance"]["build"]["commit"], "abc1234");
    assert_eq!(first["provenance"]["artefacts"][0]["uri"], "s3://rig/log1.txt");
    assert!(recs.iter().find(|r| r["scenario"] == "Two").unwrap().get("provenance").map_or(true, |p| p.is_null()));
}

#[test]
fn free_text_is_stored_without_control_characters() {
    let root = new_model();
    let json = "[{\"testCase\":\"TC-RR-001\",\"verdict\":\"waived\",\"rationale\":\"ok\\u001b[31m red\\u0007\\nnext\",\"provenance\":{\"operator\":\"al\\u001bice\"}}]";
    assert_eq!(ingest(&root, "r.json", json, &[]).2, 0);
    let sc = sidecar(&root);
    let r = &sc["records"][0];
    assert!(!r["rationale"].as_str().unwrap().chars().any(|c| c.is_control()), "{r}");
    assert!(!r["provenance"]["operator"].as_str().unwrap().chars().any(|c| c.is_control()), "{r}");
}

#[test]
fn records_from_another_rig_do_not_erase_this_one_and_replace_restores_whole_section_semantics() {
    let root = new_model();
    let bench = r#"[{"testCase":"TC-RR-001","scenario":"One","verdict":"pass","provenance":{"environment":"TE-RR-BENCH"}},
                    {"testCase":"TC-RR-001","scenario":"Two","verdict":"pass","provenance":{"environment":"TE-RR-BENCH"}}]"#;
    let hil = r#"[{"testCase":"TC-RR-001","scenario":"One","verdict":"fail","provenance":{"environment":"TE-RR-HIL"}}]"#;
    assert_eq!(ingest(&root, "bench.json", bench, &[]).2, 0);
    assert_eq!(ingest(&root, "hil.json", hil, &[]).2, 0);
    assert_eq!(sidecar(&root)["records"].as_array().unwrap().len(), 3, "bench kept beside hil");
    // re-ingesting the same key supersedes it
    let hil2 = r#"[{"testCase":"TC-RR-001","scenario":"One","verdict":"pass","provenance":{"environment":"TE-RR-HIL"}}]"#;
    assert_eq!(ingest(&root, "hil2.json", hil2, &[]).2, 0);
    let recs = sidecar(&root)["records"].clone();
    let recs = recs.as_array().unwrap();
    assert_eq!(recs.len(), 3);
    assert_eq!(recs.iter().find(|r| r["provenance"]["environment"] == "TE-RR-HIL").unwrap()["verdict"], "pass");
    // --replace drops the records the new ingest does not contain
    assert_eq!(ingest(&root, "hil3.json", hil2, &["--replace"]).2, 0);
    assert_eq!(sidecar(&root)["records"].as_array().unwrap().len(), 1);
}

#[test]
fn a_records_ingest_keeps_function_level_and_session_log_verdicts() {
    let root = new_model();
    write(&root, "junit.xml", "<testsuite><testcase classname=\"c\" name=\"t_ok\"/></testsuite>");
    let (_, err, code) = run(&root, &["ingest-results", "--format", "junit", root.join("junit.xml").to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(ingest(&root, "r.json", ALL_VERDICTS, &[]).2, 0);
    let sc = sidecar(&root);
    assert!(sc["by_leaf"].as_object().unwrap().contains_key("t_ok"), "{sc}");
    assert!(sc["records"].as_array().unwrap().len() == 6);
}

#[test]
fn the_existing_readers_still_work_after_a_records_ingest() {
    let root = new_model();
    assert_eq!(ingest(&root, "r.json", ALL_VERDICTS, &[]).2, 0);
    for cmd in [vec!["matrix"], vec!["validate"], vec!["audit"], vec!["results", "failures"]] {
        let (out, err, code) = run(&root, &cmd);
        // 1 and 2 are findings / "not ready", not failures of the tool; a crash would be a panic or another code
        assert!((0..=2).contains(&code), "{cmd:?} exited {code}: {out}{err}");
        assert!(!err.to_lowercase().contains("panick"), "{cmd:?}: {err}");
    }
}

#[test]
fn per_configuration_and_run_history_work_with_records() {
    let root = new_model();
    write(&root, "CONF-RR.md", "---\nid: CONF-RR\ntype: Configuration\nname: Conf\nfeatures: {}\n---\n\nC.\n");
    let (out, err, code) = ingest(&root, "r.json", ALL_VERDICTS, &["--run", "run-1", "--config", "CONF-RR"]);
    assert_eq!(code, 0, "{out}{err}");
    let (runs, _, _) = run(&root, &["results", "runs"]);
    assert!(runs.contains("run-1"), "{runs}");
    let sc = sidecar(&root);
    assert!(sc["by_config"]["CONF-RR"]["records"].as_array().is_some(), "{sc}");
}
