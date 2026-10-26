//! TC-TRS-DREP-001 / REQ-TRS-DREP-001 (GH #371): `syscribe design-report <part>`.
//!
//! Written before the implementation.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-drep-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("Sw/_index.md", "---\ntype: Package\nname: Sw\n---\n");
    w("REQ-001.md", "---\ntype: Requirement\nid: REQ-001\nname: r\nstatus: draft\nreqDomain: software\n---\n\nThe unit shall parse.\n");
    w("TC-001.md", "---\ntype: TestCase\nid: TC-001\nname: t\nstatus: draft\ntestLevel: L1\nverifies:\n  - REQ-001\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("Sw/Gain.md", "---\ntype: AttributeDef\nname: Gain\nrange: \"0..10\"\n---\n\nG.\n");
    w("Sw/Bus.md", "---\ntype: PortDef\nname: Bus\noperations:\n  - name: send\n    parameters:\n      - {name: frame, typedBy: ScalarValues::Integer, direction: in}\n---\n\nB.\n");
    w("Sw/Parse.md", "---\ntype: ActionDef\nname: Parse\nparameters:\n  - {name: frame, typedBy: ScalarValues::Integer, direction: in}\n---\n\nP.\n");
    w(
        "Sw/Unit.md",
        "---\ntype: PartDef\nname: Unit\ndomain: software\nlevel: unit\nstatus: approved\nimplementedBy: \"repo:src/frame.rs#parse\"\nsatisfies:\n  - REQ-001\nram: \"4 kB\"\nperiod: 10 ms\n\
operations:\n  - name: parse\n    returnType: ScalarValues::Boolean\n    realizedBy: Sw::Parse\n    parameters:\n      - {name: frame, typedBy: ScalarValues::Integer, direction: in}\n\
features:\n  - {name: bus, typedBy: Sw::Bus}\n  - {name: gain, typedBy: Sw::Gain, unit: V}\n---\n\nParses a frame.\n",
    );
    w("Hw/_index.md", "---\ntype: Package\nname: Hw\n---\n");
    w("Hw/Mcu.md", "---\ntype: PartDef\nname: Mcu\ndomain: hardware\n---\n\nM.\n");
    r
}

fn run(root: &PathBuf, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn json(root: &PathBuf, part: &str) -> serde_json::Value {
    let (o, c) = run(root, &["design-report", part, "--json"]);
    assert_eq!(c, 0, "{o}");
    serde_json::from_str(&o).unwrap_or_else(|e| panic!("{e}: {o}"))
}

#[test]
fn json_lists_every_section() {
    let v = json(&model(), "Sw::Unit");
    assert_eq!(v["element"], "Sw::Unit");
    assert_eq!(v["level"], "unit");
    assert!(v["description"].as_str().unwrap().contains("Parses a frame"));

    let iface = v["interface"].as_array().unwrap();
    let names: Vec<_> = iface.iter().map(|o| o["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"parse") && names.contains(&"send"), "{iface:?}");
    let parse = iface.iter().find(|o| o["name"] == "parse").unwrap();
    assert_eq!(parse["returnType"], "ScalarValues::Boolean");
    assert_eq!(parse["parameters"][0]["name"], "frame");

    assert_eq!(v["behaviour"], serde_json::json!(["Sw::Parse"]));

    let data = v["data"].as_array().unwrap();
    assert_eq!(data.len(), 1, "a port is interface, not data: {data:?}");
    assert_eq!(data[0]["name"], "gain");
    assert_eq!(data[0]["unit"], "V");
    assert_eq!(data[0]["range"], "0..10");

    assert_eq!(v["resources"]["ram"], "4 kB");
    assert_eq!(v["code"], serde_json::json!(["repo:src/frame.rs#parse"]));
    assert_eq!(v["requirements"], serde_json::json!(["REQ-001"]));
    let tests = v["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 1, "{tests:?}");
    assert_eq!(tests[0]["id"], "TC-001");
    assert_eq!(tests[0]["level"], "L1");
    assert!(tests[0]["verdict"].is_string());
}

#[test]
fn text_has_a_heading_per_section() {
    let (o, c) = run(&model(), &["design-report", "Sw::Unit"]);
    assert_eq!(c, 0, "{o}");
    for h in ["# Sw::Unit", "## Interface", "## Behaviour", "## Data", "## Resources", "## Code", "## Requirements", "## Tests"] {
        assert!(o.contains(h), "missing {h}: {o}");
    }
}

#[test]
fn an_empty_section_says_so() {
    let r = model();
    std::fs::write(r.join("Sw/Bare.md"), "---\ntype: PartDef\nname: Bare\ndomain: software\n---\n\nBare.\n").unwrap();
    let (o, c) = run(&r, &["design-report", "Sw::Bare"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("None."), "{o}");
}

#[test]
fn a_non_software_part_or_unknown_element_is_a_usage_error() {
    let r = model();
    for bad in ["Hw::Mcu", "Nope::X", "REQ-001"] {
        let (o, c) = run(&r, &["design-report", bad]);
        assert_eq!(c, 1, "{bad}: {o}");
    }
    let (o, c) = run(&r, &["design-report"]);
    assert_eq!(c, 1, "{o}");
}
