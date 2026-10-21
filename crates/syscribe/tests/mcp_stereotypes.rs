//! TC-TRS-META-060..064 / REQ-TRS-META-060..064 (GH #376, #377, #379): what the MCP server exposes of the
//! stereotype work — resolved applications on `get_element`, `metadata`/`where` filters on `search` and
//! `list_by_type`, the `meta` and `gates` read tools, and a clear refusal to write the built-in library.
//!
//! Written before the implementation.

mod common;
use common::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// The shared fixture plus package `Stz`: stereotypes `Safety` (integrity; level defaults to 2), `ASIL_D :> Safety`,
/// `Optional`, parts `P1` Safety(3), `P2` Safety(1)+Optional, `P3` ASIL_D(4), `P4` none, `P5` software (an `autoApply:`
/// profile gives it Optional), and requirements `REQ-STZ-001` Safety(3) with an active test, `REQ-STZ-002` Safety(3) without.
/// `toml` is appended to the fixture's `.syscribe.toml`.
fn model(toml: &str) -> PathBuf {
    model_with(toml, false)
}

/// [`model`], optionally with the built-in `StdMetadata` library enabled (`[metadata] stdlib = true`).
fn model_with(toml: &str, stdlib: bool) -> PathBuf {
    let root = fixture_copy();
    write(&root, "Stz/_index.md", "---\ntype: Package\nname: Stz\n---\n");
    write(&root, "Stz/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n  - name: level\n    typedBy: ScalarValues::Integer\n    default: 2\n---\n");
    write(&root, "Stz/ASIL_D.md", "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: Stz::Safety\n---\n");
    write(&root, "Stz/Optional.md", "---\ntype: MetadataDef\nname: Optional\n---\n");
    let part = |n: &str, extra: &str| write(&root, &format!("Stz/{n}.md"), &format!("---\ntype: PartDef\nname: {n}\n{extra}---\n\n{n}.\n"));
    part("P1", "metadata:\n  - type: Stz::Safety\n    integrity: 3\n");
    part("P2", "metadata:\n  - type: Stz::Safety\n    integrity: 1\n  - Stz::Optional\n");
    part("P3", "metadata:\n  - type: Stz::ASIL_D\n    integrity: 4\n");
    part("P4", "");
    part("P5", "domain: software\n");
    write(&root, "Stz/Prof/_index.md", "---\ntype: Package\nname: Prof\nprofile: true\nautoApply:\n  - select: {type: PartDef, where: {domain: software}}\n    apply: {type: Stz::Optional}\n---\n\nProfile.\n");
    let req = |id: &str| write(&root, &format!("Stz/{id}.md"), &format!("---\ntype: Requirement\nid: {id}\nname: r\nstatus: approved\nreqDomain: software\nmetadata:\n  - type: Stz::Safety\n    integrity: 3\n---\n\nThe system shall work.\n"));
    req("REQ-STZ-001");
    req("REQ-STZ-002");
    write(&root, "Stz/TC-STZ-001.md", "---\ntype: TestCase\nid: TC-STZ-001\nname: t\nstatus: active\ntestLevel: L2\nverifies: [REQ-STZ-001]\n---\n\n```gherkin\nFeature: f\n\n  Scenario: s\n    Given a\n    Then b\n```\n");
    let cfg = root.join(".syscribe.toml");
    let existing = std::fs::read_to_string(&cfg).unwrap_or_default();
    let std = if stdlib { "stdlib = true\n" } else { "" };
    std::fs::write(cfg, format!("{existing}\n[metadata]\nprofiles = [\"Stz::Prof\"]\n{std}{toml}\n")).unwrap();
    root
}

fn started(toml: &str) -> (Mcp, PathBuf) {
    let root = model(toml);
    let mut mcp = Mcp::start(&root);
    mcp.initialize();
    (mcp, root)
}

fn is_error(res: &Value) -> bool {
    res.get("isError").and_then(Value::as_bool) == Some(true)
}

fn error_text(res: &Value) -> String {
    res.pointer("/content/0/text").and_then(Value::as_str).unwrap_or("").to_string()
}

/// The `Stz::` qualified names in a `results`/`items` array, sorted.
fn stz(v: &Value, key: &str) -> Vec<String> {
    let mut names: Vec<String> = v[key].as_array().unwrap_or_else(|| panic!("{key} in {v}")).iter().filter_map(|r| r["qname"].as_str()).filter(|q| q.starts_with("Stz::P")).map(str::to_string).collect();
    names.sort();
    names
}

// ── REQ-TRS-META-060: get_element ──────────────────────────────────────────

#[test]
fn get_element_carries_the_resolved_applications_with_defaults() {
    let (mut mcp, _r) = started("");
    let e = mcp.call_tool("get_element", json!({"ref": "Stz::P1"}));
    let app = &e["stereotypes"][0];
    assert_eq!(app["def"], "Stz::Safety", "{e}");
    assert_eq!(app["values"]["integrity"], 3, "{e}");
    assert_eq!(app["values"]["level"], 2, "the declared default is filled in: {e}");
    assert_eq!(app["defaulted"], json!(["level"]), "{e}");
    assert!(app.get("derived").is_none_or(|d| d == false), "{e}");
}

#[test]
fn it_is_there_in_every_form_and_the_raw_metadata_stays_under_frontmatter() {
    let (mut mcp, _r) = started("");
    let detail = mcp.call_tool("get_element", json!({"ref": "Stz::P1", "detail": true}));
    assert_eq!(detail["stereotypes"][0]["def"], "Stz::Safety", "{detail}");
    assert!(detail["frontmatter"]["metadata"].is_array(), "the raw form is unchanged: {detail}");
    let projected = mcp.call_tool("get_element", json!({"ref": "Stz::P1", "fields": ["status"]}));
    assert_eq!(projected["stereotypes"][0]["def"], "Stz::Safety", "{projected}");
}

#[test]
fn a_specialisation_inherits_defaults_and_a_derived_application_is_marked() {
    let (mut mcp, _r) = started("");
    let p3 = mcp.call_tool("get_element", json!({"ref": "Stz::P3"}));
    assert_eq!(p3["stereotypes"][0]["def"], "Stz::ASIL_D", "{p3}");
    assert_eq!(p3["stereotypes"][0]["values"]["level"], 2, "inherited from Safety: {p3}");
    let p5 = mcp.call_tool("get_element", json!({"ref": "Stz::P5"}));
    assert_eq!(p5["stereotypes"][0]["def"], "Stz::Optional", "{p5}");
    assert_eq!(p5["stereotypes"][0]["derived"], true, "{p5}");
}

#[test]
fn no_applications_means_no_key() {
    let (mut mcp, _r) = started("");
    let p4 = mcp.call_tool("get_element", json!({"ref": "Stz::P4"}));
    assert!(p4.get("stereotypes").is_none(), "{p4}");
}

// ── REQ-TRS-META-061: search / list_by_type ────────────────────────────────

#[test]
fn search_filters_by_metadata_clauses() {
    let (mut mcp, _r) = started("");
    let mut q = |m: Value| stz(&mcp_search(&mut mcp, m), "results");
    assert_eq!(q(json!(["Safety"])), ["Stz::P1", "Stz::P2", "Stz::P3"], "a specialisation counts");
    assert_eq!(q(json!(["Safety", "!Optional"])), ["Stz::P1", "Stz::P3"], "all clauses must hold");
    assert_eq!(q(json!(["Optional,ASIL_D"])), ["Stz::P2", "Stz::P3", "Stz::P5"], "a comma list is any of (P5's Optional is derived)");
    assert_eq!(q(json!(["!Safety"])), ["Stz::P4", "Stz::P5"]);
}

fn mcp_search(mcp: &mut Mcp, metadata: Value) -> Value {
    mcp.call_tool("search", json!({"type": "PartDef", "metadata": metadata, "limit": 200}))
}

#[test]
fn list_by_type_filters_by_metadata_and_where_and_totals_the_filtered_set() {
    let (mut mcp, _r) = started("");
    let l = mcp.call_tool("list_by_type", json!({"type": "PartDef", "metadata": ["Safety"], "limit": 2}));
    assert_eq!(l["total"], 3, "total counts the filtered set, not the page: {l}");
    assert_eq!(l["items"].as_array().unwrap().len(), 2, "{l}");
    let w = mcp.call_tool("list_by_type", json!({"type": "PartDef", "where": "meta.Safety.integrity>=2", "limit": 100}));
    assert_eq!(stz(&w, "items"), ["Stz::P1", "Stz::P3"], "{w}");
    let both = mcp.call_tool("list_by_type", json!({"type": "PartDef", "metadata": ["Safety"], "where": "meta.Safety.integrity>=2", "limit": 100}));
    assert_eq!(stz(&both, "items"), ["Stz::P1", "Stz::P3"]);
}

#[test]
fn an_unknown_stereotype_name_is_a_tool_error_on_both_tools() {
    let (mut mcp, _r) = started("");
    for tool in ["search", "list_by_type"] {
        let r = mcp.call_tool_raw(tool, json!({"type": "PartDef", "metadata": ["Nonesuch"]}));
        assert!(is_error(&r), "{tool}: {r}");
        assert!(error_text(&r).contains("Nonesuch"), "{tool}: {r}");
    }
    let bad = mcp.call_tool_raw("list_by_type", json!({"type": "PartDef", "where": "meta.Safety.integrity=>2"}));
    assert!(is_error(&bad), "{bad}");
}

// ── REQ-TRS-META-062: meta ─────────────────────────────────────────────────

#[test]
fn the_meta_tool_returns_the_cli_report() {
    let (mut mcp, root) = started("");
    let got = mcp.call_tool("meta", json!({"def": "Safety", "group_by": "integrity"}));
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(&root)
        .args(["meta", "--def", "Safety", "--group-by", "integrity", "--json"])
        .output()
        .unwrap();
    let cli: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)));
    assert_eq!(got, cli);
    assert!(got["groups"].as_array().is_some_and(|g| g.iter().any(|x| x["value"] == "3")), "{got}");
}

#[test]
fn meta_without_a_def_counts_applications_and_an_unknown_def_is_an_error() {
    let (mut mcp, _r) = started("");
    let all = mcp.call_tool("meta", json!({}));
    assert!(all["definitions"].as_array().is_some_and(|d| d.iter().any(|x| x["def"] == "Stz::Safety")), "{all}");
    let bad = mcp.call_tool_raw("meta", json!({"def": "Nonesuch"}));
    assert!(is_error(&bad), "{bad}");
}

// ── REQ-TRS-META-063: gates ────────────────────────────────────────────────

#[test]
fn the_gates_tool_lists_what_validate_gates_on() {
    let (mut mcp, _r) = started("[gates]\ndeny_meta = [\"Safety.integrity>=4\"]\n\n[[gates.coverage]]\nselect = \"Safety where integrity >= 2\"\n");
    let g = mcp.call_tool("gates", json!({}));
    assert_eq!(g["tripped"], true, "{g}");
    let deny = g["denyMeta"].as_array().unwrap_or_else(|| panic!("{g}"));
    assert!(deny.iter().any(|d| d["condition"] == "Safety.integrity>=4" && d["element"] == "Stz::P3"), "{g}");
    let cov = g["coverage"].as_array().unwrap();
    assert_eq!(cov.len(), 1, "REQ-STZ-002 has no test: {g}");
    assert!(cov[0].as_str().unwrap().contains("coverage gate") && cov[0].as_str().unwrap().contains("REQ-STZ-002"), "{g}");
}

#[test]
fn with_no_gate_configured_nothing_is_tripped() {
    let (mut mcp, _r) = started("");
    let g = mcp.call_tool("gates", json!({}));
    assert_eq!(g["tripped"], false, "{g}");
    assert_eq!(g["denyMeta"], json!([]));
    assert_eq!(g["coverage"], json!([]));
}

#[test]
fn a_malformed_gate_is_a_tool_error() {
    let (mut mcp, _r) = started("[[gates.coverage]]\nselect = \"Nonesuch\"\n");
    let r = mcp.call_tool_raw("gates", json!({}));
    assert!(is_error(&r), "{r}");
    assert!(error_text(&r).contains("Nonesuch"), "{r}");
}

// ── REQ-TRS-META-064: the built-in library is read-only ────────────────────

#[test]
fn every_write_tool_refuses_the_built_in_library_by_name() {
    let root = model_with("", true);
    let mut mcp = Mcp::start(&root);
    mcp.initialize();
    let calls: Vec<(&str, Value)> = vec![
        ("update_element", json!({"ref": "StdMetadata::Safety", "fields": {"status": "approved"}, "dry_run": false})),
        ("delete_element", json!({"ref": "StdMetadata::Safety", "force": true, "dry_run": false})),
        ("move_element", json!({"ref": "StdMetadata::Safety", "dest": "Stz::Moved", "dry_run": false})),
        ("apply_changes", json!({"operations": [{"op": "update", "ref": "StdMetadata::Safety", "fields": {"status": "approved"}}], "dry_run": false})),
    ];
    for (tool, args) in calls {
        let r = mcp.call_tool_raw(tool, args);
        let text = format!("{} {}", error_text(&r), r);
        assert!(is_error(&r) || text.contains("built into"), "{tool}: {r}");
        assert!(text.contains("built into the tool") && text.to_lowercase().contains("read-only"), "{tool} says why: {r}");
    }
    assert!(!root.join("StdMetadata").exists() && !root.join("Stz/Moved.md").exists(), "nothing was written");
}
