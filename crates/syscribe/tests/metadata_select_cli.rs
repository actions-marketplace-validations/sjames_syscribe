//! TC-TRS-META-024, -025 / REQ-TRS-META-024, -025 (GH #376): `matrix --select`, and a stereotype
//! application that applies only in some products.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-metaselcli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
    w("B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n---\n");
    // Requirements: 001 Safety(3), 002 Safety(1), 003 none.
    let req = |id: &str, m: &str| w(&format!("R/{id}.md"), &format!("---\ntype: Requirement\nid: {id}\nname: r\nstatus: approved\nreqDomain: software\n{m}---\n\nThe system shall work.\n"));
    req("REQ-SC-001", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    req("REQ-SC-002", "metadata:\n  - type: B::Safety\n    integrity: 1\n");
    req("REQ-SC-003", "");
    // Variants: Item applies Safety only where Features::Grid is selected.
    w("Features/Grid.md", "---\ntype: FeatureDef\nid: FEAT-GRID\nname: Grid\ngroupKind: optional\n---\n\nG.\n");
    w("Configs/CONF-GRID-ON.md", "---\nid: CONF-GRID-ON\ntype: Configuration\nname: on\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Grid: true\n---\n\nC.\n");
    w("Configs/CONF-GRID-OFF.md", "---\nid: CONF-GRID-OFF\ntype: Configuration\nname: off\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Grid: false\n---\n\nC.\n");
    w("B/Item.md", "---\ntype: PartDef\nname: Item\nmetadata:\n  - type: B::Safety\n    integrity: 2\n    appliesWhen: Features::Grid\n  - type: B::Safety\n    integrity: 4\n---\n\nI.\n");
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn rows(root: &Path, args: &[&str]) -> Vec<String> {
    let (o, c) = run(root, args);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    let arr = v.as_array().cloned().or_else(|| v["rows"].as_array().cloned()).unwrap_or_else(|| panic!("{o}"));
    let mut ids: Vec<String> = arr.iter().filter_map(|r| r["id"].as_str().or_else(|| r["requirement"].as_str()).map(str::to_string)).collect();
    ids.sort();
    ids
}

// ── REQ-TRS-META-024: matrix --select ─────────────────────────────────────

#[test]
fn matrix_select_keeps_the_requirements_that_a_selector_matches() {
    let r = model();
    assert_eq!(rows(&r, &["matrix", "--json", "--select", "Safety where integrity >= 2"]), ["REQ-SC-001"]);
    assert_eq!(rows(&r, &["matrix", "--json", "--select", "Safety"]), ["REQ-SC-001", "REQ-SC-002"]);
}

#[test]
fn matrix_select_composes_with_tag_and_repeats_as_and() {
    let r = model();
    assert_eq!(rows(&r, &["matrix", "--json", "--select", "Safety", "--select", "Safety where integrity < 2"]), ["REQ-SC-002"]);
}

#[test]
fn a_malformed_select_is_a_usage_error() {
    let r = model();
    let (o, c) = run(&r, &["matrix", "--select", "Safety where integrity >="]);
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("where"), "{o}");
}

// ── REQ-TRS-META-025: appliesWhen on an application ───────────────────────

fn item(root: &Path, config: Option<&str>) -> serde_json::Value {
    let mut a = vec!["list", "PartDef", "--json"];
    if let Some(c) = config {
        a.extend(["--config", c]);
    }
    let (o, c) = run(root, &a);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    v.as_array().unwrap().iter().find(|r| r["qualifiedName"] == "B::Item").cloned().unwrap_or_else(|| panic!("B::Item in {o}"))
}

fn integrities(row: &serde_json::Value) -> Vec<i64> {
    let mut v: Vec<i64> = row["metadata"].as_array().map(|a| a.iter().filter_map(|m| m["values"]["integrity"].as_i64()).collect()).unwrap_or_default();
    v.sort();
    v
}

#[test]
fn without_a_configuration_every_application_is_present() {
    assert_eq!(integrities(&item(&model(), None)), [2, 4]);
}

#[test]
fn an_application_whose_guard_is_false_is_dropped_under_a_configuration() {
    let r = model();
    assert_eq!(integrities(&item(&r, Some("CONF-GRID-ON"))), [2, 4]);
    assert_eq!(integrities(&item(&r, Some("CONF-GRID-OFF"))), [4], "the guarded application is not in this product");
}

#[test]
fn applies_when_is_not_a_tagged_value() {
    let r = model();
    let (o, c) = run(&r, &["validate"]);
    assert_eq!(c, 0, "{o}");
    assert!(!o.contains("W045"), "{o}");
    let app = &item(&r, None)["metadata"][0];
    assert!(app["values"].get("appliesWhen").is_none(), "{app}");
}

#[test]
fn diff_reports_an_application_that_differs_between_products() {
    let r = model();
    let (o, c) = run(&r, &["diff", "--config", "CONF-GRID-ON", "--config", "CONF-GRID-OFF", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    let d = v["metadata"].as_array().unwrap_or_else(|| panic!("a metadata section: {v}")).iter().find(|x| x["element"] == "B::Item").unwrap_or_else(|| panic!("{v}"));
    assert_eq!(d["onlyInA"][0], "B::Safety", "{d}");
}
