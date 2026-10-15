//! TC-TRS-RESJOIN-001 / REQ-TRS-RESJOIN-001: a result is joined to its TestCase by id, qualified name or extRef,
//! unmatched records are reported, and a result older than a change to its test or requirement is flagged.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

const REQ: &str = "---\nid: REQ-JN-001\ntype: Requirement\nname: \"R\"\nstatus: approved\nreqDomain: system\n---\n\nThe system shall do the thing.\n";

fn tc(id: &str, ext: &str, body_extra: &str) -> String {
    format!("---\nid: {id}\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: \"{id}\"\nverifies: [REQ-JN-001]\n{ext}---\n\n```gherkin\nFeature: f\n  Scenario: One\n    Given x\n    Then y\n```\n{body_extra}")
}

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-resjoin-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Join\n---\n\nRoot.\n");
    write(&root, "REQ-JN-001.md", REQ);
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", ""));
    write(&root, "Sub/_index.md", "---\ntype: Package\nname: Sub\n---\n\nSub.\n");
    write(&root, "Sub/TC-JN-002.md", &tc("TC-JN-002", "", ""));
    write(&root, "TC-JN-003.md", &tc("TC-JN-003", "extRef: \"RIG:SYS-012\"\n", ""));
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

fn ingest(root: &Path, json: &str, extra: &[&str]) -> (String, String, i32) {
    write(root, "r.json", json);
    let file = root.join("r.json");
    let mut args = vec!["ingest-results", "--format", "records"];
    args.extend_from_slice(extra);
    args.push(file.to_str().unwrap());
    run(root, &args)
}

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

fn trace(root: &Path) -> String {
    run(root, &["trace", "REQ-JN-001"]).0
}

fn validate(root: &Path, extra: &[&str]) -> (String, i32) {
    let mut args = vec!["validate"];
    args.extend_from_slice(extra);
    let (o, _, c) = run(root, &args);
    (o, c)
}

fn pass_all(ids: &[&str]) -> String {
    let recs: Vec<String> = ids.iter().map(|i| format!(r#"{{"testCase":"{i}","scenario":"One","verdict":"pass"}}"#)).collect();
    format!("[{}]", recs.join(","))
}

// ── join ─────────────────────────────────────────────────────────────────────

#[test]
fn a_record_attaches_by_id_by_qualified_name_and_by_extref_and_is_stored_by_id() {
    let root = new_model();
    let (qn, _, _) = run(&root, &["show", "TC-JN-002"]);
    let qname = qn.lines().find_map(|l| l.strip_prefix("# ")).unwrap_or("TC-JN-002").trim().to_string();
    let (out, err, code) = ingest(&root, &pass_all(&["TC-JN-001", &qname, "RIG:SYS-012"]), &[]);
    assert_eq!(code, 0, "{out}{err}");
    let t = trace(&root);
    for id in ["TC-JN-001", "TC-JN-002", "TC-JN-003"] {
        assert!(t.contains(&format!("{id} [pass]")), "{id} should be joined:\n{t}");
    }
    let sc = sidecar(&root);
    let mut stored: Vec<&str> = sc["records"].as_array().unwrap().iter().map(|r| r["testCase"].as_str().unwrap()).collect();
    stored.sort();
    assert_eq!(stored, ["TC-JN-001", "TC-JN-002", "TC-JN-003"], "stored under the canonical id, whatever the report called them");
    assert!(sc.get("unmatched").is_none() || sc["unmatched"].as_array().unwrap().is_empty());
}

#[test]
fn the_same_test_named_two_ways_is_one_record() {
    let root = new_model();
    let json = r#"[{"testCase":"TC-JN-003","scenario":"One","verdict":"fail"},{"testCase":"RIG:SYS-012","scenario":"One","verdict":"pass"}]"#;
    assert_eq!(ingest(&root, json, &[]).2, 0);
    let recs = sidecar(&root)["records"].clone();
    assert_eq!(recs.as_array().unwrap().len(), 1, "{recs}");
    assert_eq!(recs[0]["verdict"], "pass", "the later one wins");
}

#[test]
fn an_ambiguous_extref_is_an_error_naming_the_candidates_and_writes_nothing() {
    let root = new_model();
    write(&root, "TC-JN-004.md", &tc("TC-JN-004", "extRef: \"RIG:SYS-012\"\n", ""));
    let (_, err, code) = ingest(&root, &pass_all(&["RIG:SYS-012"]), &[]);
    assert_ne!(code, 0);
    assert!(err.contains("TC-JN-003") && err.contains("TC-JN-004") && err.contains("ambiguous"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
}

#[test]
fn a_key_naming_something_that_is_not_a_testcase_is_an_error() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &pass_all(&["REQ-JN-001"]), &[]);
    assert_ne!(code, 0);
    assert!(err.contains("not a TestCase"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
}

#[test]
fn one_bad_key_rejects_the_whole_file_even_beside_good_ones() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &pass_all(&["TC-JN-001", "REQ-JN-001"]), &[]);
    assert_ne!(code, 0);
    assert!(err.contains("REQ-JN-001"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists(), "never half-stored");
}

// ── unmatched ────────────────────────────────────────────────────────────────

#[test]
fn unmatched_records_are_counted_listed_and_kept_apart_from_the_evidence() {
    let root = new_model();
    let (out, err, code) = ingest(&root, &pass_all(&["TC-JN-001", "TC-NOPE-1", "TC-NOPE-2"]), &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("2 of 3 record(s) name no TestCase"), "{out}");
    assert!(out.contains("TC-NOPE-1") && out.contains("TC-NOPE-2"), "{out}");
    let sc = sidecar(&root);
    assert_eq!(sc["records"].as_array().unwrap().len(), 1, "only the joined record is evidence");
    assert_eq!(sc["unmatched"].as_array().unwrap().len(), 2);
    let (failures, _, _) = run(&root, &["results", "failures"]);
    assert!(failures.contains("Unmatched records (2)") && failures.contains("TC-NOPE-1"), "{failures}");
    let (json, _, _) = run(&root, &["results", "failures", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["unmatched"].as_array().unwrap().len(), 2);
}

#[test]
fn strict_rejects_a_file_with_an_unmatched_record_and_writes_nothing() {
    let root = new_model();
    let (_, err, code) = ingest(&root, &pass_all(&["TC-JN-001", "TC-NOPE-1"]), &["--strict"]);
    assert_eq!(code, 1);
    assert!(err.contains("TC-NOPE-1") && err.contains("nothing was written"), "{err}");
    assert!(!root.join(".syscribe/results.json").exists());
    // a clean file passes under --strict
    assert_eq!(ingest(&root, &pass_all(&["TC-JN-001"]), &["--strict"]).2, 0);
}

#[test]
fn strict_is_refused_for_the_other_formats() {
    let root = new_model();
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    let (_, err, code) = run(&root, &["ingest-results", "--strict", root.join("j.xml").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("--strict"), "{err}");
}

#[test]
fn the_unmatched_list_is_that_of_the_latest_ingest() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-NOPE-1"]), &[]);
    assert_eq!(sidecar(&root)["unmatched"].as_array().unwrap().len(), 1);
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    let sc = sidecar(&root);
    assert!(sc.get("unmatched").is_none() || sc["unmatched"].as_array().unwrap().is_empty(), "{sc}");
}

#[test]
fn many_unmatched_keys_are_summarised_not_dumped() {
    let root = new_model();
    let ids: Vec<String> = (0..40).map(|i| format!("TC-NOPE-{i}")).collect();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let (out, _, code) = ingest(&root, &pass_all(&refs), &[]);
    assert_eq!(code, 0);
    assert!(out.contains("and 35 more"), "{out}");
    assert!(!out.contains("TC-NOPE-39"), "{out}");
}

// ── staleness ────────────────────────────────────────────────────────────────

fn w091(root: &Path) -> Vec<String> {
    validate(root, &[]).0.lines().filter(|l| l.contains("W091")).map(String::from).collect()
}

#[test]
fn a_fresh_result_is_silent() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    assert!(w091(&root).is_empty(), "{:?}", w091(&root));
    assert!(!trace(&root).contains("[stale]"));
}

#[test]
fn editing_the_testcase_after_the_result_raises_w091_and_marks_it_stale() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", "\nA new paragraph that changes what the test checks.\n"));
    let w = w091(&root);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("TC-JN-001.md") && w[0].contains("the TestCase TC-JN-001 changed"), "{w:?}");
    assert!(trace(&root).contains("TC-JN-001 [pass] [stale]"), "{}", trace(&root));
}

#[test]
fn editing_a_verified_requirement_after_the_result_raises_w091() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001", "TC-JN-003"]), &[]);
    write(&root, "REQ-JN-001.md", &REQ.replace("do the thing", "do the other thing"));
    let w = w091(&root);
    assert_eq!(w.len(), 2, "both tests verify the changed requirement: {w:?}");
    assert!(w.iter().all(|l| l.contains("requirement REQ-JN-001 changed")), "{w:?}");
}

#[test]
fn a_requirement_that_vanished_is_reported_as_gone() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    std::fs::remove_file(root.join("REQ-JN-001.md")).unwrap();
    let w = w091(&root);
    assert!(w.iter().any(|l| l.contains("no longer exists")), "{w:?}");
}

#[test]
fn editorial_changes_do_not_make_a_result_stale() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    // the name is editorial, and an ordinary status change is workflow, as for suspect links
    write(&root, "REQ-JN-001.md", &REQ.replace("name: \"R\"", "name: \"A better label\"").replace("status: approved", "status: implemented"));
    assert!(w091(&root).is_empty(), "{:?}", w091(&root));
}

#[test]
fn deny_makes_a_stale_result_fail_the_gate_and_reingest_clears_it() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", "\nChanged.\n"));
    let (_, code) = validate(&root, &["--deny", "W091"]);
    assert_ne!(code, 0, "--deny W091 must fail");
    let (_, plain) = validate(&root, &[]);
    assert_eq!(plain, 0, "a warning alone does not fail validate");
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    assert!(w091(&root).is_empty(), "re-ingesting the re-run result clears it");
    assert_eq!(validate(&root, &["--deny", "W091"]).1, 0);
}

#[test]
fn a_draft_testcase_is_not_reported() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", "\nChanged.\n").replace("status: active", "status: draft"));
    assert!(w091(&root).is_empty());
}

#[test]
fn a_sidecar_written_without_hashes_stays_silent() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    let mut sc = sidecar(&root);
    for r in sc["records"].as_array_mut().unwrap() {
        r.as_object_mut().unwrap().remove("tcHash");
        r.as_object_mut().unwrap().remove("reqHashes");
    }
    std::fs::write(root.join(".syscribe/results.json"), serde_json::to_string(&sc).unwrap()).unwrap();
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", "\nChanged.\n"));
    assert!(w091(&root).is_empty());
}

#[test]
fn hashes_a_report_claims_are_discarded() {
    let root = new_model();
    let json = r#"[{"testCase":"TC-JN-001","scenario":"One","verdict":"pass","tcHash":"blake3:forged","reqHashes":{"REQ-JN-001":"blake3:forged"}}]"#;
    assert_eq!(ingest(&root, json, &[]).2, 0);
    assert!(w091(&root).is_empty(), "a producer cannot vouch for freshness, nor forge staleness");
    let r = sidecar(&root)["records"][0].clone();
    assert!(r["tcHash"].as_str().unwrap().starts_with("blake3:") && r["tcHash"] != "blake3:forged", "{r}");
}

#[test]
fn staleness_does_not_change_the_verdict() {
    let root = new_model();
    ingest(&root, &pass_all(&["TC-JN-001"]), &[]);
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", "\nChanged.\n"));
    assert!(trace(&root).contains("TC-JN-001 [pass]"), "{}", trace(&root));
}

#[test]
fn stale_results_of_a_configuration_are_found_too() {
    let root = new_model();
    write(&root, "CONF-JN.md", "---\nid: CONF-JN\ntype: Configuration\nname: Conf\nfeatures: {}\n---\n\nC.\n");
    assert_eq!(ingest(&root, &pass_all(&["TC-JN-001"]), &["--config", "CONF-JN"]).2, 0);
    write(&root, "TC-JN-001.md", &tc("TC-JN-001", "", "\nChanged.\n"));
    assert_eq!(w091(&root).len(), 1, "{:?}", w091(&root));
}

#[test]
fn a_large_model_joins_quickly() {
    let root = new_model();
    for i in 0..300 {
        write(&root, &format!("Bulk/TC-BULK-{i:03}.md"), &tc(&format!("TC-BULK-{i:03}"), &format!("extRef: \"BULK:{i}\"\n"), ""));
    }
    write(&root, "Bulk/_index.md", "---\ntype: Package\nname: Bulk\n---\n\nBulk.\n");
    let ids: Vec<String> = (0..300).map(|i| format!("BULK:{i}")).collect();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let t = std::time::Instant::now();
    let (_, err, code) = ingest(&root, &pass_all(&refs), &[]);
    assert_eq!(code, 0, "{err}");
    assert!(t.elapsed().as_secs() < 30, "joining 300 records by extRef took {:?}", t.elapsed());
    assert_eq!(sidecar(&root)["records"].as_array().unwrap().len(), 300);
}

#[test]
fn the_testcase_template_shows_the_extref_join_key() {
    let root = new_model();
    let (text, err, _) = run(&root, &["template", "TestCase"]);
    assert!(!text.is_empty(), "no template printed: {err}");
    assert!(text.contains("# extRef:") && text.contains("ingest-results --format records"), "{text}");
}
