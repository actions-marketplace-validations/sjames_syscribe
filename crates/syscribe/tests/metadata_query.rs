//! TC-TRS-META-020, -021, -022 / REQ-TRS-META-020..022 (GH #376): selecting elements by
//! stereotype and tagged value.
//!
//! Written before the implementation; every test fails until `--where meta.…` and the
//! multi-definition / negated `--metadata` forms land.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// Four parts: `P1`..`P3` apply `Safety` (integrity 2, 3, 1; P3 also `Optional`), `P4` applies
/// nothing. `Regulatory` is applied to `P1` only. `Safety.level` defaults to 2 where unwritten.
fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-metaquery-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("B/_index.md", "---\ntype: Package\nname: B\n---\n");
    w("B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n  - name: goal\n    typedBy: ScalarValues::String\n  - name: level\n    typedBy: ScalarValues::Integer\n    default: 2\n---\n");
    w("B/Optional.md", "---\ntype: MetadataDef\nname: Optional\n---\n");
    w("B/Regulatory.md", "---\ntype: MetadataDef\nname: Regulatory\nfeatures:\n  - name: status\n    typedBy: ScalarValues::String\n---\n");
    w("B/P1.md", "---\ntype: PartDef\nname: P1\nmetadata:\n  - type: B::Safety\n    integrity: 2\n    goal: SG-1\n  - type: B::Regulatory\n    status: unverified\n---\n");
    w("B/P2.md", "---\ntype: PartDef\nname: P2\nmetadata:\n  - type: B::Safety\n    integrity: 3\n    level: 4\n---\n");
    w("B/P3.md", "---\ntype: PartDef\nname: P3\nmetadata:\n  - type: B::Safety\n    integrity: 1\n  - B::Optional\n---\n");
    w("B/P4.md", "---\ntype: PartDef\nname: P4\n---\n");
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

/// The parts `list PartDef <args>` returns, as sorted short names.
fn parts(root: &Path, args: &[&str]) -> Vec<String> {
    let mut a = vec!["list", "PartDef", "--json"];
    a.extend_from_slice(args);
    let (o, c) = run(root, &a);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"));
    let mut names: Vec<String> = v.as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect();
    names.sort();
    names
}

// ── REQ-TRS-META-020 ───────────────────────────────────────────────────────

#[test]
fn a_comparison_on_a_tagged_value_selects_exactly_the_matching_parts() {
    let r = model();
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity>=2"]), ["P1", "P2"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity>2"]), ["P2"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity<=1"]), ["P3"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity<3"]), ["P1", "P3"]);
}

#[test]
fn equality_inequality_and_regex_forms() {
    let r = model();
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity=3"]), ["P2"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.goal=SG-1"]), ["P1"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.goal=~^SG-"]), ["P1"]);
    assert_eq!(parts(&r, &["--where", "meta.Regulatory.status=unverified"]), ["P1"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity!=3"]), ["P1", "P3"], "an element without the stereotype is not matched by !=");
}

#[test]
fn presence_of_a_value_and_of_the_stereotype() {
    let r = model();
    assert_eq!(parts(&r, &["--where", "meta.Safety.goal"]), ["P1"], "presence of a tagged value");
    assert_eq!(parts(&r, &["--where", "meta.Safety"]), ["P1", "P2", "P3"], "presence of the stereotype");
}

#[test]
fn an_element_without_the_stereotype_is_not_matched_and_is_not_an_error() {
    let r = model();
    let (o, c) = run(&r, &["list", "PartDef", "--where", "meta.Regulatory.status=verified"]);
    assert_eq!(c, 0, "{o}");
    assert!(!o.contains("P4"), "{o}");
}

#[test]
fn defaults_take_part() {
    let r = model();
    // P1 and P3 do not write `level`; the declared default (2) applies. P2 writes 4.
    assert_eq!(parts(&r, &["--where", "meta.Safety.level=2"]), ["P1", "P3"]);
    assert_eq!(parts(&r, &["--where", "meta.Safety.level>=2"]), ["P1", "P2", "P3"]);
}

#[test]
fn predicates_are_anded_and_combine_with_custom_fields() {
    let r = model();
    assert_eq!(parts(&r, &["--where", "meta.Safety.integrity>=2", "--where", "meta.Regulatory.status=unverified"]), ["P1"]);
}

#[test]
fn a_definition_is_matched_by_last_segment_or_qualified_name() {
    let r = model();
    assert_eq!(parts(&r, &["--where", "meta.B::Safety.integrity=3"]), ["P2"]);
}

#[test]
fn find_and_ls_accept_the_same_predicate() {
    let r = model();
    let (o, c) = run(&r, &["find", "P", "--where", "meta.Safety.integrity>=3"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("P2") && !o.contains("P1") && !o.contains("P3"), "{o}");
    let (o, c) = run(&r, &["ls", "B", "--where", "meta.Safety.integrity>=3"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("P2") && !o.contains("P1") && !o.contains("P4"), "{o}");
}

#[test]
fn a_malformed_meta_predicate_names_the_supported_forms() {
    let r = model();
    let (o, c) = run(&r, &["list", "PartDef", "--where", "meta.Safety.integrity=>2"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("meta.<Def>.<key>"), "{o}");
    let (o, c) = run(&r, &["list", "PartDef", "--where", "meta."]);
    assert_ne!(c, 0, "{o}");
}

#[test]
fn a_comparison_against_a_non_number_is_an_error_not_a_silent_empty_result() {
    let r = model();
    let (o, c) = run(&r, &["list", "PartDef", "--where", "meta.Safety.integrity>=high"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("number"), "{o}");
}

// ── REQ-TRS-META-021 ───────────────────────────────────────────────────────

#[test]
fn repeating_metadata_requires_every_definition() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "Safety", "--metadata", "Optional"]), ["P3"]);
}

#[test]
fn a_comma_list_accepts_either_definition() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "Optional,Regulatory"]), ["P1", "P3"]);
}

#[test]
fn a_leading_bang_negates_a_definition() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "!Safety"]), ["P4"]);
    assert_eq!(parts(&r, &["--metadata", "Safety", "--metadata", "!Optional"]), ["P1", "P2"]);
}

#[test]
fn an_unknown_definition_is_reported_not_silently_empty() {
    let r = model();
    let (o, c) = run(&r, &["list", "PartDef", "--metadata", "Nonesuch"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("Nonesuch"), "{o}");
}

#[test]
fn the_single_definition_form_is_unchanged() {
    let r = model();
    assert_eq!(parts(&r, &["--metadata", "Safety"]), ["P1", "P2", "P3"]);
}

// ── REQ-TRS-META-022 ───────────────────────────────────────────────────────

#[test]
fn applications_are_structured_in_json_and_absent_when_there_are_none() {
    let r = model();
    let (o, _) = run(&r, &["list", "PartDef", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    let row = |n: &str| v.as_array().unwrap().iter().find(|r| r["name"] == n).unwrap().clone();
    let p1 = row("P1");
    let safety = p1["metadata"].as_array().unwrap().iter().find(|a| a["def"] == "B::Safety").unwrap();
    assert_eq!(safety["values"]["integrity"], 2);
    assert_eq!(safety["values"]["goal"], "SG-1");
    assert_eq!(safety["defaulted"][0], "level");
    assert!(row("P4").get("metadata").is_none(), "no empty key for an element without applications");
}
