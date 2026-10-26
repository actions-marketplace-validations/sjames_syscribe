//! TC-TRS-RESPLUG-001 / REQ-TRS-RESPLUG-001: a results plugin converts a foreign test report into normalised
//! result records. Real subprocesses: each plugin is a small shell script in the model root.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("syscribe-resplug-{}-{}", std::process::id(), n)).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Plug\n---\n\nRoot.\n");
    write(&root, "REQ-PL-001.md", "---\nid: REQ-PL-001\ntype: Requirement\nname: R\nstatus: approved\nreqDomain: system\n---\n\nBody.\n");
    write(
        &root,
        "TC-PL-001.md",
        "---\nid: TC-PL-001\ntype: TestCase\ntestLevel: L5\nstatus: active\nname: T\nverifies: [REQ-PL-001]\nextRef: \"RIG:PL-1\"\n---\n\n```gherkin\nFeature: f\n  Scenario: One\n    Given x\n    Then y\n```\n",
    );
    write(&root, "TE-PL-001.md", "---\nid: TE-PL-001\ntype: TestEnvironment\nname: Rig\nstatus: available\nenvironmentKind: hil\n---\n\nRig.\n");
    write(&root, "report.txt", "rig log\nTEST TC-PL-001 One PASS\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// Install a plugin `alias` whose command is the shell script `body`, with `extra` lines in its toml table.
fn plugin(root: &Path, alias: &str, body: &str, extra: &str) {
    let script = root.join(format!("{alias}.sh"));
    std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let toml = root.join(".syscribe.toml");
    let mut text = std::fs::read_to_string(&toml).unwrap_or_default();
    text.push_str(&format!("\n[plugins.{alias}]\ncommand = \"./{alias}.sh\"\nkind = \"results\"\ntimeout_ms = 5000\n{extra}\n"));
    std::fs::write(toml, text).unwrap();
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().expect("spawn syscribe");
    (String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned(), out.status.code().unwrap_or(-1))
}

fn ingest(root: &Path, alias: &str, extra: &[&str]) -> (String, String, i32) {
    let file = root.join("report.txt");
    let fmt = format!("plugin:{alias}");
    let mut args = vec!["ingest-results", "--format", fmt.as_str()];
    args.extend_from_slice(extra);
    args.push(file.to_str().unwrap());
    run(root, &args)
}

fn sidecar(root: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap()
}

const OK: &str = r#"cat > request.json; printf '%s' '{"records":[{"testCase":"TC-PL-001","scenario":"One","verdict":"pass"}],"diagnostics":["converted 1 test"]}'"#;

fn nothing_written(root: &Path) {
    assert!(!root.join(".syscribe/results.json").exists(), "nothing may be written");
}

// ── the happy path and the protocol ──────────────────────────────────────────

#[test]
fn a_configured_plugin_ingests_a_report_exactly_as_the_built_in_format_would() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    let (out, err, code) = ingest(&root, "rig", &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("records via plugin rig"), "{out}");
    assert!(err.contains("results plugin 'rig': converted 1 test"), "diagnostics are relayed: {err}");
    let via_plugin = sidecar(&root)["records"].clone();

    let root2 = new_model();
    write(&root2, "direct.json", r#"[{"testCase":"TC-PL-001","scenario":"One","verdict":"pass"}]"#);
    let (_, e, c) = run(&root2, &["ingest-results", "--format", "records", root2.join("direct.json").to_str().unwrap()]);
    assert_eq!(c, 0, "{e}");
    let mut a = via_plugin;
    let mut b = sidecar(&root2)["records"].clone();
    for v in [&mut a, &mut b] {
        for r in v.as_array_mut().unwrap() {
            r.as_object_mut().unwrap().remove("tcHash");
            r.as_object_mut().unwrap().remove("reqHashes");
        }
    }
    assert_eq!(a, b, "the same stored records");
    let (trace, _, _) = run(&root, &["trace", "REQ-PL-001"]);
    assert!(trace.contains("TC-PL-001 [pass]"), "{trace}");
}

#[test]
fn the_request_carries_the_documented_fields_and_only_the_given_options() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    assert_eq!(ingest(&root, "rig", &[]).2, 0);
    let req: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join("request.json")).unwrap()).unwrap();
    assert_eq!(req["protocolVersion"], 1);
    assert_eq!(req["alias"], "rig");
    assert!(Path::new(req["reportPath"].as_str().unwrap()).is_absolute() && req["reportPath"].as_str().unwrap().ends_with("report.txt"));
    assert!(Path::new(req["modelRoot"].as_str().unwrap()).is_absolute());
    for absent in ["run", "config", "environment"] {
        assert!(req.get(absent).is_none(), "{absent} is only sent when given: {req}");
    }
    write(&root, "CONF-PL.md", "---\nid: CONF-PL\ntype: Configuration\nname: C\nfeatures: {}\n---\n\nC.\n");
    assert_eq!(ingest(&root, "rig", &["--run", "r7", "--config", "CONF-PL", "--environment", "TE-PL-001"]).2, 0);
    let req: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join("request.json")).unwrap()).unwrap();
    assert_eq!((req["run"].as_str(), req["config"].as_str(), req["environment"].as_str()), (Some("r7"), Some("CONF-PL"), Some("TE-PL-001")));
}

#[test]
fn a_relative_report_path_is_sent_as_an_absolute_one() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    let (_, err, code) = run(&root, &["ingest-results", "--format", "plugin:rig", "report.txt"]);
    // the file is relative to the current directory, not the model: the tool must not guess
    if code == 0 {
        let req: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join("request.json")).unwrap()).unwrap();
        assert!(Path::new(req["reportPath"].as_str().unwrap()).is_absolute());
    } else {
        assert!(err.contains("report.txt"), "{err}");
    }
}

#[test]
fn a_plugin_that_ignores_its_stdin_still_works() {
    let root = new_model();
    plugin(&root, "rig", r#"printf '%s' '{"records":[{"testCase":"TC-PL-001","verdict":"pass"}]}'"#, "");
    let (_, err, code) = ingest(&root, "rig", &[]);
    assert_eq!(code, 0, "{err}");
}

#[test]
fn noise_on_stderr_with_a_clean_exit_is_fine() {
    let root = new_model();
    plugin(&root, "rig", r#"echo 'warming up' >&2; printf '%s' '{"records":[{"testCase":"TC-PL-001","verdict":"pass"}]}'"#, "");
    assert_eq!(ingest(&root, "rig", &[]).2, 0);
}

#[test]
fn the_same_report_gives_the_same_records_every_time() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    ingest(&root, "rig", &[]);
    let first = sidecar(&root)["records"].clone();
    ingest(&root, "rig", &[]);
    assert_eq!(first, sidecar(&root)["records"]);
}

// ── failures are findings, and nothing is written ────────────────────────────

#[test]
fn a_plugin_that_cannot_be_used_is_reported_with_its_alias_and_cause() {
    let cases: Vec<(&str, String, &str)> = vec![
        ("missing", String::new(), "not found"),
        ("nonzero", "echo 'boom: bad log line 3' >&2; exit 3".to_string(), "boom: bad log line 3"),
        ("garbage", "echo 'this is not json'".to_string(), "invalid envelope"),
        ("empty", "printf ''".to_string(), "invalid envelope"),
        ("notarray", r#"printf '%s' '{"records": {"testCase": "x"}}'"#.to_string(), "must be an array"),
        ("typo", r#"printf '%s' '{"record": []}'"#.to_string(), "invalid envelope"),
        ("none", r#"printf '%s' '{"records": []}'"#.to_string(), "no records"),
        ("toplevelarray", r#"printf '%s' '[{"testCase":"TC-PL-001","verdict":"pass"}]'"#.to_string(), "invalid envelope"),
    ];
    for (alias, body, needle) in cases {
        let root = new_model();
        if alias == "missing" {
            write(&root, ".syscribe.toml", "[plugins.missing]\ncommand = \"./nope.sh\"\nkind = \"results\"\n");
        } else {
            plugin(&root, alias, &body, "");
        }
        let (_, err, code) = ingest(&root, alias, &[]);
        assert_ne!(code, 0, "{alias}");
        assert!(err.contains("E553") && err.contains(alias), "{alias}: {err}");
        assert!(err.to_lowercase().contains(needle), "{alias} should mention '{needle}': {err}");
        nothing_written(&root);
    }
}

#[test]
fn a_plugin_that_hangs_is_killed_at_its_timeout() {
    let root = new_model();
    plugin(&root, "slow", "sleep 30", "");
    let toml = std::fs::read_to_string(root.join(".syscribe.toml")).unwrap().replace("timeout_ms = 5000", "timeout_ms = 300");
    std::fs::write(root.join(".syscribe.toml"), toml).unwrap();
    let t = std::time::Instant::now();
    let (_, err, code) = ingest(&root, "slow", &[]);
    assert_ne!(code, 0);
    assert!(t.elapsed().as_secs() < 10, "{:?}", t.elapsed());
    assert!(err.contains("timeout_ms"), "{err}");
    nothing_written(&root);
}

#[test]
fn an_unknown_alias_and_an_empty_alias_are_errors() {
    let root = new_model();
    let (_, err, code) = ingest(&root, "ghost", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("E553") && err.contains("[plugins.ghost]"), "{err}");
    let (_, err, code) = run(&root, &["ingest-results", "--format", "plugin:", root.join("report.txt").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("Unknown --format"), "{err}");
    nothing_written(&root);
}

#[test]
fn a_missing_report_is_an_error_before_the_plugin_runs() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    let (_, err, code) = run(&root, &["ingest-results", "--format", "plugin:rig", root.join("absent.txt").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("absent.txt"), "{err}");
    assert!(!root.join("request.json").exists(), "the plugin must not have been started");
}

// ── a plugin cannot bypass the validator ─────────────────────────────────────

#[test]
fn what_a_plugin_returns_is_validated_like_the_built_in_format() {
    let bad = [
        (r#"{"testCase":"TC-PL-001","verdict":"waived"}"#, "rationale"),
        (r#"{"testCase":"TC-PL-001","verdict":"great"}"#, "great"),
        (r#"{"testCase":"TC-PL-001","verdict":"pass","unknownField":1}"#, "unknownField"),
        (r#"{"testCase":"TC-PL-001","verdict":"pass","provenance":{"environment":"TE-NOPE"}}"#, "TE-NOPE"),
        (r#"{"testCase":"TC-PL-001","verdict":"pass","measurements":[{"name":"m","value":"9 ms","limit":{"op":"<","value":"5 ms"}}]}"#, "limit"),
        (r#"{"testCase":"REQ-PL-001","verdict":"pass"}"#, "not a TestCase"),
    ];
    for (rec, needle) in bad {
        let root = new_model();
        plugin(&root, "rig", &format!("printf '%s' '{{\"records\":[{rec}]}}'"), "");
        let (_, err, code) = ingest(&root, "rig", &[]);
        assert_ne!(code, 0, "{rec} was accepted");
        assert!(err.contains(needle), "{rec}: {err}");
        nothing_written(&root);
    }
}

#[test]
fn a_plugin_that_forges_hashes_cannot() {
    let root = new_model();
    plugin(&root, "rig", r#"printf '%s' '{"records":[{"testCase":"TC-PL-001","verdict":"pass","tcHash":"blake3:forged"}]}'"#, "");
    assert_eq!(ingest(&root, "rig", &[]).2, 0);
    let r = sidecar(&root)["records"][0].clone();
    assert_ne!(r["tcHash"], "blake3:forged");
}

#[test]
fn the_join_applies_to_plugin_records_too() {
    let root = new_model();
    plugin(&root, "rig", r#"printf '%s' '{"records":[{"testCase":"RIG:PL-1","scenario":"One","verdict":"pass"},{"testCase":"RIG:NOPE","verdict":"fail"}]}'"#, "");
    let (out, err, code) = ingest(&root, "rig", &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("1 of 2 record(s) name no TestCase"), "{out}");
    let (_, err, code) = ingest(&root, "rig", &["--strict"]);
    assert_eq!(code, 1, "--strict applies: {err}");
}

// ── command-line values ──────────────────────────────────────────────────────

#[test]
fn command_line_values_fill_provenance_a_record_leaves_empty_and_never_override() {
    let root = new_model();
    plugin(
        &root,
        "rig",
        r#"printf '%s' '{"records":[{"testCase":"TC-PL-001","scenario":"One","verdict":"pass"},{"testCase":"TC-PL-001","verdict":"pass","provenance":{"build":{"commit":"own111"}}}]}'"#,
        "",
    );
    let (_, err, code) = ingest(&root, "rig", &["--environment", "TE-PL-001", "--build-commit", "cli999"]);
    assert_eq!(code, 0, "{err}");
    let recs = sidecar(&root)["records"].clone();
    let by_scn = |scn: bool| recs.as_array().unwrap().iter().find(|r| r.get("scenario").is_some() == scn).unwrap().clone();
    assert_eq!(by_scn(true)["provenance"]["build"]["commit"], "cli999");
    assert_eq!(by_scn(true)["provenance"]["environment"], "TE-PL-001");
    assert_eq!(by_scn(false)["provenance"]["build"]["commit"], "own111", "a record's own value is kept");
    assert_eq!(by_scn(false)["provenance"]["environment"], "TE-PL-001");
}

#[test]
fn build_commit_works_for_the_built_in_records_format_and_is_refused_elsewhere() {
    let root = new_model();
    write(&root, "d.json", r#"[{"testCase":"TC-PL-001","verdict":"pass"}]"#);
    let (_, err, code) = run(&root, &["ingest-results", "--format", "records", "--build-commit", "abc", root.join("d.json").to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(sidecar(&root)["records"][0]["provenance"]["build"]["commit"], "abc");
    write(&root, "j.xml", "<testsuite><testcase classname=\"c\" name=\"t\"/></testsuite>");
    let (_, err, code) = run(&root, &["ingest-results", "--build-commit", "abc", root.join("j.xml").to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(err.contains("--build-commit"), "{err}");
}

#[test]
fn replace_run_and_config_work_with_a_plugin() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    write(&root, "CONF-PL.md", "---\nid: CONF-PL\ntype: Configuration\nname: C\nfeatures: {}\n---\n\nC.\n");
    assert_eq!(ingest(&root, "rig", &["--run", "r1"]).2, 0);
    assert_eq!(ingest(&root, "rig", &["--config", "CONF-PL"]).2, 0);
    assert!(sidecar(&root)["by_config"]["CONF-PL"]["records"].as_array().is_some());
    let (runs, _, _) = run(&root, &["results", "runs"]);
    assert!(runs.contains("r1"), "{runs}");
    assert_eq!(ingest(&root, "rig", &["--replace"]).2, 0);
}

// ── kinds do not mix ─────────────────────────────────────────────────────────

#[test]
fn a_foreign_format_plugin_cannot_be_used_for_results() {
    let root = new_model();
    plugin(&root, "dsl", OK, "");
    let toml = std::fs::read_to_string(root.join(".syscribe.toml")).unwrap().replace("kind = \"results\"\n", "");
    std::fs::write(root.join(".syscribe.toml"), toml).unwrap();
    let (_, err, code) = ingest(&root, "dsl", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("E552") && err.contains("not a results plugin"), "{err}");
    nothing_written(&root);
}

#[test]
fn a_results_plugin_cannot_ingest_a_foreign_format_package() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    write(&root, "Legacy/_index.md", "---\ntype: Package\nname: Legacy\nforeignFormat: rig\n---\n\nLegacy.\n");
    let (out, _, _) = run(&root, &["validate"]);
    assert!(out.lines().any(|l| l.contains("E552") && l.contains("kind")), "{out}");
}

#[test]
fn an_unknown_kind_is_rejected_for_both_uses() {
    let root = new_model();
    plugin(&root, "odd", OK, "");
    let toml = std::fs::read_to_string(root.join(".syscribe.toml")).unwrap().replace("kind = \"results\"", "kind = \"mystery\"");
    std::fs::write(root.join(".syscribe.toml"), toml).unwrap();
    let (_, err, code) = ingest(&root, "odd", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("E552") && err.contains("mystery"), "{err}");
    write(&root, "Legacy/_index.md", "---\ntype: Package\nname: Legacy\nforeignFormat: odd\n---\n\nLegacy.\n");
    assert!(run(&root, &["validate"]).0.contains("E552"));
}

#[test]
fn an_explicit_foreign_kind_still_works_as_a_foreign_format_plugin() {
    let root = new_model();
    plugin(&root, "dsl", r#"printf '%s' '{"elements":[],"diagnostics":[]}'"#, "");
    let toml = std::fs::read_to_string(root.join(".syscribe.toml")).unwrap().replace("kind = \"results\"", "kind = \"foreign\"");
    std::fs::write(root.join(".syscribe.toml"), toml).unwrap();
    write(&root, "Legacy/_index.md", "---\ntype: Package\nname: Legacy\nforeignFormat: dsl\n---\n\nLegacy.\n");
    let (out, _, _) = run(&root, &["validate"]);
    assert!(!out.contains("E552") && !out.contains("E551"), "{out}");
}

// ── dry run ──────────────────────────────────────────────────────────────────

#[test]
fn the_dry_run_prints_the_raw_envelope_and_merges_nothing() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    let (out, err, code) = run(&root, &["plugins", "run", "rig", "--report", root.join("report.txt").to_str().unwrap(), "--dry-run"]);
    assert_eq!(code, 0, "{err}");
    let env: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(env["records"][0]["testCase"], "TC-PL-001");
    nothing_written(&root);
}

#[test]
fn the_dry_run_shows_what_a_broken_plugin_wrote_without_validating_it() {
    let root = new_model();
    plugin(&root, "rig", r#"printf '%s' '{"records":[{"verdict":"bogus"}]}'"#, "");
    let (out, _, code) = run(&root, &["plugins", "run", "rig", "--report", root.join("report.txt").to_str().unwrap(), "--dry-run"]);
    assert_eq!(code, 0);
    assert!(out.contains("bogus"), "{out}");
}

#[test]
fn the_dry_run_needs_a_report_for_a_results_plugin_and_refuses_one_for_a_foreign_plugin() {
    let root = new_model();
    plugin(&root, "rig", OK, "");
    let (_, err, code) = run(&root, &["plugins", "run", "rig", "--dry-run"]);
    assert_ne!(code, 0);
    assert!(err.contains("--report"), "{err}");
    // a foreign-format plugin takes no report
    plugin(&root, "dsl", "echo '{}'", "");
    let toml = std::fs::read_to_string(root.join(".syscribe.toml")).unwrap();
    let toml = toml.replacen("[plugins.dsl]\ncommand = \"./dsl.sh\"\nkind = \"results\"", "[plugins.dsl]\ncommand = \"./dsl.sh\"\nkind = \"foreign\"", 1);
    std::fs::write(root.join(".syscribe.toml"), toml).unwrap();
    write(&root, "Legacy/_index.md", "---\ntype: Package\nname: Legacy\nforeignFormat: dsl\n---\n\nLegacy.\n");
    let (_, err, code) = run(&root, &["plugins", "run", "dsl", "--report", root.join("report.txt").to_str().unwrap(), "--dry-run"]);
    assert_ne!(code, 0);
    assert!(err.contains("--report") && err.contains("foreign-format"), "{err}");
    // and without --report it is still the foreign-format dry run
    assert_eq!(run(&root, &["plugins", "run", "dsl", "--dry-run"]).2, 0);
}

// ── robustness ───────────────────────────────────────────────────────────────

#[test]
fn a_very_large_output_does_not_deadlock() {
    let root = new_model();
    // 5000 records, about a megabyte, produced by the shell itself
    plugin(
        &root,
        "big",
        r#"printf '{"records":['; i=0; while [ $i -lt 5000 ]; do [ $i -gt 0 ] && printf ','; printf '{"testCase":"TC-PL-001","scenario":"S%d","verdict":"pass","rationale":null}' $i; i=$((i+1)); done; printf ']}'"#,
        "",
    );
    let t = std::time::Instant::now();
    let (_, err, code) = ingest(&root, "big", &[]);
    assert_eq!(code, 0, "{err}");
    assert!(t.elapsed().as_secs() < 30, "{:?}", t.elapsed());
    assert_eq!(sidecar(&root)["records"].as_array().unwrap().len(), 5000);
}

#[test]
fn a_plugin_that_exits_zero_after_a_failure_message_but_wrote_nothing_is_an_error() {
    let root = new_model();
    plugin(&root, "rig", "echo 'could not parse the log' >&2", "");
    let (_, err, code) = ingest(&root, "rig", &[]);
    assert_ne!(code, 0);
    assert!(err.contains("E553"), "{err}");
}

#[test]
fn diagnostics_may_be_strings_or_objects_with_a_message() {
    let root = new_model();
    plugin(&root, "rig", r#"printf '%s' '{"records":[{"testCase":"TC-PL-001","verdict":"pass"}],"diagnostics":["plain note",{"severity":"warning","message":"object note"}]}'"#, "");
    let (_, err, code) = ingest(&root, "rig", &[]);
    assert_eq!(code, 0, "{err}");
    assert!(err.contains("plain note") && err.contains("object note"), "{err}");
}

// ── the worked example ───────────────────────────────────────────────────────

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).unwrap();
        }
    }
}

#[test]
fn the_worked_example_behaves_as_its_readme_says() {
    if Command::new("python3").arg("--version").output().is_err() {
        eprintln!("python3 not available; skipping the example");
        return;
    }
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/stdio-plugins/results-python");
    let tmp = std::env::temp_dir().join(format!("syscribe-resplug-example-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    copy_dir(&src, &tmp);
    let model = tmp.join("model");
    let rig = tmp.join("rig.log");
    let tap = tmp.join("run.tap");

    let (out, err, code) = run(&model, &["plugins", "run", "rig", "--report", rig.to_str().unwrap(), "--dry-run"]);
    assert_eq!(code, 0, "{err}");
    assert!(serde_json::from_str::<serde_json::Value>(&out).unwrap()["records"].as_array().unwrap().len() == 3);

    let (out, err, code) = run(&model, &["ingest-results", "--format", "plugin:rig", "--environment", "TE-HIL-001", "--build-commit", "zzz", rig.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("RIG:UNKNOWN-9"), "the unknown test is reported: {out}");
    let sc = sidecar(&model);
    let recs = sc["records"].as_array().unwrap();
    assert_eq!(recs.len(), 2);
    assert!(recs.iter().all(|r| r["testCase"] == "TC-BRK-001" && r["provenance"]["environment"] == "TE-HIL-001"), "{recs:?}");
    assert!(recs.iter().all(|r| r["provenance"]["build"]["commit"] == "abc1234"), "the log's own build wins over --build-commit");
    let timed = recs.iter().find(|r| r["scenario"] == "Request reaches the actuator in time").unwrap();
    assert_eq!(timed["measurements"][0]["name"], "latency");

    let (_, err, code) = run(&model, &["ingest-results", "--format", "plugin:tap", "--environment", "TE-HIL-001", tap.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let (trace, _, _) = run(&model, &["trace", "REQ-BRK-001"]);
    assert!(trace.contains("TC-BRK-001 [fail] @ TE-HIL-001") && trace.contains("TC-BRK-002 [pass]"), "{trace}");
    let _ = std::fs::remove_dir_all(&tmp);
}
