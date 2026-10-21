//! TC-TRS-META-050, -051, -052, -054 / REQ-TRS-META-050..052, -054 (GH #379): stereotype-conditioned
//! checks, gates, the `meta` report and the baseline condition.
//!
//! Written before the implementation; every test fails until it lands.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct M {
    root: PathBuf,
}

impl M {
    fn new(toml: &str) -> M {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-metachecks-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
        let m = M { root };
        m.w("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
        if !toml.is_empty() {
            m.w(".syscribe.toml", toml);
        }
        m
    }
    fn w(&self, rel: &str, c: &str) -> &M {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
        self
    }
    fn run(&self, args: &[&str]) -> (String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
    }
}

const CHECK_WARN: &str = "checks:\n  - expr: 'status == \"verified\" or reviewer != \"\"'\n    message: an unverified value needs a reviewer\n    severity: warning\n";

fn def(checks: &str) -> String {
    format!("---\ntype: MetadataDef\nname: Regulatory\nfeatures:\n  - name: status\n    typedBy: ScalarValues::String\n    enumValues: [verified, unverified, waived]\n  - name: reviewer\n    typedBy: ScalarValues::String\n{checks}---\n")
}

fn part(m: &M, name: &str, status_line: &str, app: &str) {
    m.w(&format!("B/{name}.md"), &format!("---\ntype: PartDef\nname: {name}\n{status_line}metadata:\n  - type: B::Regulatory\n{app}---\n\n{name}.\n"));
}

/// `R1` unverified/no reviewer, `R2` unverified with a reviewer, `R3` verified, `R4` like R1 but `draft`.
fn model(checks: &str, toml: &str) -> M {
    let m = M::new(toml);
    m.w("B/Regulatory.md", &def(checks));
    part(&m, "R1", "", "    status: unverified\n");
    part(&m, "R2", "", "    status: unverified\n    reviewer: jane\n");
    part(&m, "R3", "", "    status: verified\n");
    part(&m, "R4", "status: draft\n", "    status: unverified\n");
    m
}

// ── REQ-TRS-META-050 ───────────────────────────────────────────────────────

#[test]
fn a_failing_check_names_the_definition_and_the_element_and_passing_ones_are_quiet() {
    let m = model(CHECK_WARN, "");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "a warning does not fail validate: {o}");
    let w: Vec<&str> = o.lines().filter(|l| l.contains("W650")).collect();
    assert_eq!(w.len(), 1, "{o}");
    assert!(w[0].contains("R1") && w[0].contains("Regulatory") && w[0].contains("needs a reviewer"), "{}", w[0]);
}

#[test]
fn an_error_severity_check_is_e650_and_fails_validate() {
    let m = model(&CHECK_WARN.replace("warning", "error"), "");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.lines().any(|l| l.contains("E650") && l.contains("R1")), "{o}");
}

#[test]
fn a_check_without_severity_is_an_error_and_a_bare_string_is_accepted() {
    let m = model("checks:\n  - 'status == \"verified\" or reviewer != \"\"'\n", "");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("E650"), "{o}");
}

#[test]
fn a_warning_check_is_suppressed_on_a_draft_carrier_but_an_error_is_not() {
    let m = model(CHECK_WARN, "");
    let (o, _) = m.run(&["validate"]);
    assert!(!o.lines().any(|l| l.contains("W650") && l.contains("R4")), "draft suppression: {o}");
    let e = model(&CHECK_WARN.replace("warning", "error"), "");
    let (o, _) = e.run(&["validate"]);
    assert!(o.lines().any(|l| l.contains("E650") && l.contains("R4")), "{o}");
}

#[test]
fn a_specialisation_runs_the_inherited_checks() {
    let m = model(CHECK_WARN, "");
    m.w("B/Strict.md", "---\ntype: MetadataDef\nname: Strict\nsupertype: B::Regulatory\n---\n");
    m.w("B/R5.md", "---\ntype: PartDef\nname: R5\nmetadata:\n  - type: B::Strict\n    status: unverified\n---\n\nR5.\n");
    let (o, _) = m.run(&["validate"]);
    assert!(o.lines().any(|l| l.contains("W650") && l.contains("R5")), "{o}");
}

#[test]
fn a_check_in_a_disabled_profile_does_not_run() {
    let m = M::new("");
    m.w("Prof/_index.md", "---\ntype: Package\nname: Prof\nprofile: true\n---\n\nP.\n");
    m.w("Prof/Regulatory.md", &def(&CHECK_WARN.replace("warning", "error")));
    m.w("B/R1.md", "---\ntype: PartDef\nname: R1\nmetadata:\n  - type: Prof::Regulatory\n    status: unverified\n---\n\nR1.\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 0, "{o}");
    let on = M::new("[metadata]\nprofiles = [\"Prof\"]\n");
    on.w("Prof/_index.md", "---\ntype: Package\nname: Prof\nprofile: true\n---\n\nP.\n");
    on.w("Prof/Regulatory.md", &def(&CHECK_WARN.replace("warning", "error")));
    on.w("B/R1.md", "---\ntype: PartDef\nname: R1\nmetadata:\n  - type: Prof::Regulatory\n    status: unverified\n---\n\nR1.\n");
    assert_eq!(on.run(&["validate"]).1, 1);
}

#[test]
fn a_malformed_check_is_reported_on_the_definition() {
    let m = model("checks:\n  - expr: 'status =='\n    severity: sideways\n  - message: no expression\n", "");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 1, "{o}");
    let l: Vec<&str> = o.lines().filter(|l| l.contains("E650") && l.contains("Regulatory.md")).collect();
    assert!(l.len() >= 3, "unparsable expr, bad severity, missing expr: {o}");
}

// ── REQ-TRS-META-051 ───────────────────────────────────────────────────────

#[test]
fn deny_meta_fails_exactly_when_a_matching_element_exists() {
    let m = model("", "");
    let (o, c) = m.run(&["validate", "--deny-meta", "Regulatory.status=unverified"]);
    assert_eq!(c, 2, "a tripped gate is exit 2: {o}");
    let gate: Vec<&str> = o.lines().filter(|l| l.starts_with("deny-meta")).collect();
    for e in ["B::R1", "B::R2", "B::R4"] {
        assert!(gate.iter().any(|l| l.ends_with(e)), "{e} listed: {o}");
    }
    assert_eq!(gate.len(), 3, "R3 is verified: {o}");
    let (o, c) = m.run(&["validate", "--deny-meta", "Regulatory.status=waived"]);
    assert_eq!(c, 0, "{o}");
}

#[test]
fn deny_meta_takes_comparisons_and_repeats() {
    let m = model("", "");
    assert_eq!(m.run(&["validate", "--deny-meta", "Regulatory.status=waived", "--deny-meta", "Regulatory.reviewer=jane"]).1, 2);
    assert_eq!(m.run(&["validate", "--deny-meta", "Regulatory"]).1, 2, "the stereotype itself");
}

#[test]
fn a_gates_table_applies_the_same_conditions() {
    let m = model("", "[gates]\ndeny_meta = [\"Regulatory.status=unverified\"]\n");
    let (o, c) = m.run(&["validate"]);
    assert_eq!(c, 2, "{o}");
    assert!(o.contains("R1"), "{o}");
}

#[test]
fn a_malformed_deny_meta_is_a_usage_error() {
    let m = model("", "");
    let (o, c) = m.run(&["validate", "--deny-meta", "Regulatory.status=>x"]);
    assert_eq!(c, 1, "{o}");
}

// ── REQ-TRS-META-052 ───────────────────────────────────────────────────────

#[test]
fn the_meta_report_groups_a_definition_by_a_tagged_value() {
    let m = model("", "");
    let (o, c) = m.run(&["meta", "--def", "Regulatory", "--group-by", "status", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    let group = |val: &str| -> Vec<String> {
        let g = v["groups"].as_array().unwrap().iter().find(|g| g["value"] == val).unwrap_or_else(|| panic!("{val} in {v}"));
        let mut n: Vec<String> = g["elements"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        n.sort();
        n
    };
    assert_eq!(group("unverified"), ["B::R1", "B::R2", "B::R4"]);
    assert_eq!(group("verified"), ["B::R3"]);
    assert_eq!(v["groups"][0]["value"], "unverified", "largest group first");
}

#[test]
fn the_meta_report_in_text_lists_counts() {
    let m = model("", "");
    let (o, c) = m.run(&["meta", "--def", "Regulatory", "--group-by", "status"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("unverified") && o.contains('3') && o.contains("verified"), "{o}");
}

#[test]
fn the_meta_report_without_options_counts_applications_per_definition() {
    let m = model("", "");
    let (o, _) = m.run(&["meta", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let d = v["definitions"].as_array().unwrap().iter().find(|d| d["def"] == "B::Regulatory").unwrap();
    assert_eq!(d["count"], 4, "{v}");
}

#[test]
fn the_meta_report_counts_a_defaulted_value_and_an_unset_one() {
    let m = M::new("");
    m.w("B/Regulatory.md", "---\ntype: MetadataDef\nname: Regulatory\nfeatures:\n  - name: status\n    typedBy: ScalarValues::String\n    default: unverified\n  - name: reviewer\n    typedBy: ScalarValues::String\n---\n");
    part(&m, "R1", "", "");
    part(&m, "R2", "", "    status: verified\n");
    let (o, _) = m.run(&["meta", "--def", "Regulatory", "--group-by", "status", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert!(v["groups"].as_array().unwrap().iter().any(|g| g["value"] == "unverified"), "the default groups R1: {v}");
    let (o, _) = m.run(&["meta", "--def", "Regulatory", "--group-by", "reviewer", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert!(v["groups"].as_array().unwrap().iter().any(|g| g["value"].is_null()), "an unset value is its own group: {v}");
}

#[test]
fn the_meta_report_rejects_an_unknown_definition() {
    let m = model("", "");
    let (o, c) = m.run(&["meta", "--def", "Nonesuch"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("Nonesuch"), "{o}");
}

// ── REQ-TRS-META-054 ───────────────────────────────────────────────────────

fn git(dir: &Path, args: &[&str]) {
    assert!(Command::new("git").arg("-C").arg(dir).args(args).output().unwrap().status.success(), "git {args:?}");
}

fn git_model(toml: &str) -> (PathBuf, M) {
    let m = model("", toml);
    let repo = m.root.parent().unwrap().to_path_buf();
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "t@t"]);
    git(&repo, &["config", "user.name", "t"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "init"]);
    (repo, m)
}

#[test]
fn baseline_create_refuses_while_a_gate_condition_holds_and_names_the_element() {
    let (_repo, m) = git_model("[gates]\ndeny_meta = [\"Regulatory.status=unverified\"]\n");
    let (o, c) = m.run(&["baseline", "create", "--tag", "REL-2026-07"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("R1") && o.contains("Regulatory.status=unverified"), "{o}");
    assert!(!m.root.join("Baselines").exists(), "nothing was written");
}

#[test]
fn baseline_create_seals_once_the_condition_no_longer_holds() {
    let (repo, m) = git_model("[gates]\ndeny_meta = [\"Regulatory.status=unverified\"]\n");
    for n in ["R1", "R2", "R4"] {
        let p = m.root.join(format!("B/{n}.md"));
        let s = std::fs::read_to_string(&p).unwrap().replace("status: unverified", "status: verified");
        std::fs::write(p, s).unwrap();
    }
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "verify"]);
    let (o, c) = m.run(&["baseline", "create", "--tag", "REL-2026-07"]);
    assert_eq!(c, 0, "{o}");
}
