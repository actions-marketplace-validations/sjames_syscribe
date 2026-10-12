//! CLI fixes from the documentation review: unknown options rejected on every
//! option-taking command, ReqIF export well-formedness / scope, enumerated-value
//! messages, FEAT-id lookup, `impact --kinds`, `trade-study --config`, and
//! `suspect accept` on sheet-row sources.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static N: AtomicU64 = AtomicU64::new(0);

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn model() -> PathBuf {
    let root = std::env::temp_dir().join(format!("syscribe-reviewfix-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    write(&root, "_index.md", "---\ntype: Package\nname: RF\n---\n\nFixture.\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n\nFeatures.\n");
    write(&root, "Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-RF-OPT\nname: Opt\nmandatory: false\n---\n\nOptional.\n");
    write(
        &root,
        "Requirements/REQ-RF-001.md",
        "---\ntype: Requirement\nid: REQ-RF-001\nname: \"Mixed inline\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nA `Baseline` is **a `BL` element** and **`x` bold-code** with `**stars**`.\n",
    );
    root
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned(), o.status.code().unwrap_or(-1))
}

#[test]
fn every_option_taking_command_rejects_an_unknown_option() {
    let root = model();
    let cmds: &[&[&str]] = &[
        &["audit"], &["types"], &["untyped"], &["sysml"], &["link-types"], &["export-html"], &["matrix"], &["testplan"],
        &["metrics"], &["cyber-risk"], &["co-analysis"], &["safety-case"], &["zones"], &["conduits"], &["reviews"],
        &["review", "RR-1"], &["magicgrid"], &["trade-study"], &["feature-check"], &["features"], &["feature", "Features::Opt"],
        &["why-active", "Features::Opt"], &["configure", "CONF-X"], &["diff"], &["applies-when", "Features::Opt"],
        &["claim", "PI-1"], &["release", "PI-1"], &["move", "A", "B"], &["set", "A", "status=draft"],
        &["scaffold-gherkin", "TC-1"], &["plantuml"], &["repos"], &["export-reqif"], &["next-id", "REQ"],
        &["check-ref", "X"], &["path-for", "X"], &["suspect", "list"], &["baseline", "list"],
        &["fmea", "report"], &["hara", "matrix"], &["fault-tree", "render", "FT-1"], &["results", "runs"], &["refs", "X"],
        &["impact", "X"], &["build-config"], &["clusters"],
    ];
    for c in cmds {
        let mut args: Vec<&str> = c.to_vec();
        args.push("--bogus-flag-xyz");
        let (_, err, code) = run(&root, &args);
        assert_eq!(code, 1, "{c:?}: expected exit 1, stderr: {err}");
        assert!(err.contains("--bogus-flag-xyz") && err.contains("valid options"), "{c:?}: {err}");
    }
}

#[test]
fn export_reqif_is_well_formed_and_round_trips() {
    let root = model();
    let out = root.parent().unwrap().join("out.reqif");
    let (_, err, code) = run(&root, &["export-reqif", "--output", out.to_str().unwrap()]);
    assert_eq!(code, 0, "{err}");
    let x = std::fs::read_to_string(&out).unwrap();
    assert!(x.contains("<xhtml:strong>a <xhtml:code>BL</xhtml:code> element</xhtml:strong>"), "{x}");
    assert!(x.contains("<xhtml:code>**stars**</xhtml:code>"), "{x}");
    let (o, err, code) = run(&root, &["import-reqif", out.to_str().unwrap(), "--dry-run"]);
    assert_eq!(code, 0, "{o}{err}");
    assert!(!o.contains("malformed") && !err.contains("malformed"), "{o}{err}");
}

#[test]
fn export_reqif_unknown_scope_is_a_usage_error() {
    let root = model();
    let out = root.parent().unwrap().join("o.reqif");
    let (_, err, code) = run(&root, &["export-reqif", "--scope", "Nope", "--output", out.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(err.contains("Nope"), "{err}");
    assert!(!out.exists());
}

#[test]
fn cyber_risk_bad_format_lists_every_format() {
    let (_, err, code) = run(&model(), &["cyber-risk", "--format", "xml"]);
    assert_eq!(code, 1);
    for f in ["md", "html", "json", "dot", "mermaid", "plantuml", "svg"] {
        assert!(err.contains(f), "{f} missing: {err}");
    }
}

#[test]
fn feature_accepts_a_feat_id() {
    let (o, err, code) = run(&model(), &["feature", "FEAT-RF-OPT"]);
    assert_eq!(code, 0, "{err}");
    assert!(o.contains("Features::Opt"), "{o}");
    let (_, _, code) = run(&model(), &["feature", "FEAT-NOPE"]);
    assert_eq!(code, 1);
}

#[test]
fn impact_unknown_kind_is_a_usage_error() {
    let (_, err, code) = run(&model(), &["impact", "REQ-RF-001", "--kinds", "bogus"]);
    assert_eq!(code, 1);
    assert!(err.contains("bogus") && err.contains("models"), "{err}");
    let (_, _, code) = run(&model(), &["impact", "REQ-RF-001", "--kinds", "derivedFrom,models"]);
    assert_eq!(code, 0);
}

#[test]
fn trade_study_unknown_config_is_a_usage_error() {
    let (_, err, code) = run(&model(), &["trade-study", "--config", "NOPE"]);
    assert_eq!(code, 1);
    assert!(err.contains("NOPE"), "{err}");
}

#[test]
fn suspect_accept_refuses_sheet_row_sources_and_does_not_count_them() {
    let root = model();
    write(&root, "Security/_index.md", "---\ntype: Package\nname: Security\n---\n\nSecurity.\n");
    write(
        &root,
        "Safety/SG-RF-001.md",
        "---\ntype: SafetyGoal\nid: SG-RF-001\nname: \"Goal\"\nstatus: draft\nasil: B\n---\n\nGoal.\n",
    );
    write(
        &root,
        "Security/TARA-RF-001.md",
        "---\ntype: TARASheet\nid: TARA-RF-001\nname: \"TARA\"\nstatus: draft\ndamageTable:\n  - id: DS-RF-001\n    name: \"Damage\"\n    damageSeverity: severe\n    impactCategories: [safety]\n    hazardRef: SG-RF-001\n---\n\nSheet.\n",
    );
    let before = std::fs::read_to_string(root.join("Security/TARA-RF-001.md")).unwrap();
    let (o, err, code) = run(&root, &["suspect", "accept", "--all-unbaselined"]);
    assert_eq!(code, 0, "{err}");
    assert!(err.contains("DS-RF-001") && err.contains("row"), "{err}");
    assert!(!o.contains("baselined 1 "), "must not count the row link: {o}");
    // Second run reports the same skip and finds nothing new to do (stable output).
    let (o2, _, _) = run(&root, &["suspect", "accept", "--all-unbaselined"]);
    assert_eq!(o, o2);
    let (_, err, code) = run(&root, &["suspect", "accept", "DS-RF-001", "SG-RF-001"]);
    assert_eq!(code, 1);
    assert!(err.contains("row"), "{err}");
    assert_eq!(before, std::fs::read_to_string(root.join("Security/TARA-RF-001.md")).unwrap());
}

#[test]
fn feature_check_summary_prints_counts_not_findings() {
    let root = model();
    let (plain, _, _) = run(&root, &["feature-check"]);
    assert!(plain.contains("| Code | File | Message |"), "plain output lists findings:\n{plain}");
    let (summary, _, _) = run(&root, &["feature-check", "--summary"]);
    assert!(summary.contains("| Code | Severity | Count |"), "--summary prints counts:\n{summary}");
    assert!(!summary.contains("| Code | File | Message |"), "--summary replaces the findings table:\n{summary}");
    assert!(summary.contains("| W024 | warning | 1 |"), "one orphan feature counted:\n{summary}");
}
