//! TC-TRS-ARCH-001..005 / REQ-TRS-ARCH-001..005 (GH #369, slice one): `ArchitectureRule` elements with
//! `forbid-dependency` and `independence`, evaluated by `validate` over connections, typedBy and allocations.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::validator::validate;
use syscribe_model::walker::walk_model;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-arch-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn port(name: &str, dir: &str) -> String {
    format!("  - {{name: {name}, type: Port, direction: {dir}}}\n")
}

/// A trip path (`Hw::Sensor` → `Hw::Actuator`, tag `trip`), an MCU with firmware allocated to it, and a supervisor.
/// `connections` is the YAML list body of `Sys::Top`'s `connections:`; `rule` the front matter body of the rule.
fn model(connections: &str, rule: &str) -> Vec<RawElement> {
    model_with(connections, rule, &[], "")
}

/// [`model`] with extra files and extra `features:` lines (YAML) on `Sys::Top`.
fn model_with(connections: &str, rule: &str, extra: &[(&str, &str)], top_features: &str) -> Vec<RawElement> {
    let r = tempdir();
    for (p, c) in extra {
        write(&r, p, c);
    }
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for p in ["Hw", "Sw", "Sys", "Rules"] {
        write(&r, &format!("{p}/_index.md"), &format!("---\ntype: Package\nname: {p}\n---\n"));
    }
    let part = |path: &str, name: &str, domain: &str, tags: &str, extra: &str, ports: &str| {
        write(&r, path, &format!("---\ntype: PartDef\nname: {name}\ndomain: {domain}\n{tags}{extra}features:\n{ports}---\n\n{name}.\n"))
    };
    part("Hw/Sensor.md", "Sensor", "hardware", "tags: [trip]\n", "", &format!("{}{}", port("o", "out"), port("cfg", "in")));
    part("Hw/Actuator.md", "Actuator", "hardware", "tags: [trip]\n", "", &format!("{}{}{}", port("i", "in"), port("i2", "in"), port("cmd", "in")));
    part("Hw/Mcu.md", "Mcu", "hardware", "", "", &format!("{}{}", port("o", "out"), port("i", "in")));
    part("Sw/Fw.md", "Fw", "software", "", "allocatedTo: [Hw::Mcu]\n", &port("o", "out"));
    part("Sw/Sup.md", "Sup", "software", "", "", &format!("{}{}", port("o", "out"), port("i", "in")));
    write(
        &r,
        "Sys/Top.md",
        &format!(
            "---\ntype: PartDef\nname: Top\ndomain: system\nfeatures:\n  - {{name: sensor, typedBy: Hw::Sensor}}\n  - {{name: actuator, typedBy: Hw::Actuator}}\n  - {{name: mcu, typedBy: Hw::Mcu}}\n  - {{name: fw, typedBy: Sw::Fw}}\n  - {{name: sup, typedBy: Sw::Sup}}\n{top_features}connections:\n{connections}---\n\nTop.\n"
        ),
    );
    write(&r, "Rules/AR-ARC-001.md", &format!("---\ntype: ArchitectureRule\nid: AR-ARC-001\nname: the rule\n{rule}---\n\nWhy.\n"));
    walk_model(&r).unwrap()
}

const SENSE: &str = "  - {from: sensor.o, to: actuator.i}\n";
const FORBID_SW: &str = "kind: forbid-dependency\nfrom: {tag: trip}\nto: {domain: software}\n";

fn codes(els: &[RawElement], code: &str) -> Vec<(String, String)> {
    validate(els).findings.iter().filter(|f| f.code == code).map(|f| (f.file.clone(), f.message.clone())).collect()
}

fn in_order(msg: &str, parts: &[&str]) -> bool {
    let mut at = 0;
    parts.iter().all(|p| msg[at..].find(p).map(|i| at += i + p.len()).is_some())
}

// ── REQ-TRS-ARCH-004: forbid-dependency ────────────────────────────────────

#[test]
fn a_trip_path_with_no_software_dependency_passes() {
    let els = model(SENSE, FORBID_SW);
    assert!(codes(&els, "E660").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn a_direct_dependency_on_software_is_found_with_its_path() {
    let els = model(&format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n"), FORBID_SW);
    let e = codes(&els, "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].0.ends_with("AR-ARC-001.md"), "reported on the rule: {e:?}");
    assert!(e[0].1.contains("AR-ARC-001") && in_order(&e[0].1, &["Hw::Actuator", "Sw::Sup"]), "{e:?}");
}

#[test]
fn a_chain_of_two_connections_is_found_and_removing_the_last_link_clears_it() {
    let chain = format!("{SENSE}  - {{from: sup.o, to: mcu.i}}\n  - {{from: mcu.o, to: actuator.i2}}\n");
    let e = codes(&model(&chain, FORBID_SW), "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(in_order(&e[0].1, &["Hw::Actuator", "Hw::Mcu", "Sw::Sup"]), "the whole path: {e:?}");
    let cut = format!("{SENSE}  - {{from: sup.o, to: mcu.i}}\n");
    assert!(codes(&model(&cut, FORBID_SW), "E660").is_empty());
}

#[test]
fn the_direction_matters_a_producer_does_not_depend_on_its_consumer() {
    // the actuator feeds the supervisor: the supervisor depends on the actuator, not the other way round
    let els = model(&format!("{SENSE}  - {{from: actuator.cmd, to: sup.i}}\n"), FORBID_SW);
    assert!(codes(&els, "E660").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn via_restricts_which_edge_sources_count() {
    let wired = format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n");
    let only_alloc = model(&wired, &format!("{FORBID_SW}via: [allocations]\n"));
    assert!(codes(&only_alloc, "E660").is_empty(), "the connection is ignored");
    // software depends on the hardware it is allocated to, never the reverse
    let fw_on_mcu = model(SENSE, "kind: forbid-dependency\nfrom: {domain: software}\nto: {domain: hardware}\nvia: [allocations]\n");
    let e = codes(&fw_on_mcu, "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(in_order(&e[0].1, &["Sw::Fw", "Hw::Mcu"]), "{e:?}");
    // a composite depends on the definitions of its usages
    let contains = model(SENSE, "kind: forbid-dependency\nfrom: {package: Sys}\nto: {domain: software}\nvia: [typedBy]\n");
    assert!(!codes(&contains, "E660").is_empty());
    let not_contains = model(SENSE, "kind: forbid-dependency\nfrom: {package: Sys}\nto: {domain: software}\nvia: [connections]\n");
    assert!(codes(&not_contains, "E660").is_empty());
}

#[test]
fn severity_warning_is_w660_and_a_draft_rule_is_not_evaluated() {
    let wired = format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n");
    let w = model(&wired, &format!("{FORBID_SW}severity: warning\n"));
    assert_eq!(codes(&w, "W660").len(), 1);
    assert!(codes(&w, "E660").is_empty());
    let d = model(&wired, &format!("{FORBID_SW}status: draft\n"));
    assert!(codes(&d, "E660").is_empty() && codes(&d, "W660").is_empty());
}

// ── REQ-TRS-ARCH-005: independence ─────────────────────────────────────────

const INDEP: &str = "kind: independence\nfrom: {tag: trip}\nto: {domain: software}\n";

#[test]
fn two_sets_that_share_nothing_are_independent() {
    // the firmware is allocated to the MCU; nothing on the trip path touches the MCU
    let els = model(SENSE, INDEP);
    assert!(codes(&els, "E661").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn a_shared_element_is_reported_with_a_path_from_each_side() {
    // the sensor is configured by the MCU, on which the firmware runs
    let els = model(&format!("{SENSE}  - {{from: mcu.o, to: sensor.cfg}}\n"), INDEP);
    let e = codes(&els, "E661");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].0.ends_with("AR-ARC-001.md"));
    assert!(e[0].1.contains("Hw::Mcu"), "names the shared component: {e:?}");
    assert!(in_order(&e[0].1, &["Hw::Sensor", "Hw::Mcu"]) && in_order(&e[0].1, &["Sw::Fw", "Hw::Mcu"]), "a path from each set: {e:?}");
}

#[test]
fn removing_the_sharing_makes_the_rule_pass() {
    assert!(codes(&model(SENSE, INDEP), "E661").is_empty());
}

#[test]
fn wiring_one_set_to_a_member_of_the_other_shares_that_member() {
    let els = model(&format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n"), INDEP);
    let e = codes(&els, "E661");
    assert!(e.iter().any(|(_, m)| m.contains("Sw::Sup")), "the supervisor is in both closures: {e:?}");
}

#[test]
fn independence_severity_warning_is_w661() {
    let els = model(&format!("{SENSE}  - {{from: mcu.o, to: sensor.cfg}}\n"), &format!("{INDEP}severity: warning\n"));
    assert_eq!(codes(&els, "W661").len(), 1);
    assert!(codes(&els, "E661").is_empty());
}

// ── REQ-TRS-ARCH-001, -002: the element and its selectors ──────────────────

#[test]
fn a_well_formed_rule_and_a_stereotype_selector_validate_clean() {
    let els = model(SENSE, FORBID_SW);
    assert!(codes(&els, "E662").is_empty() && codes(&els, "W047").is_empty(), "{:?}", validate(&els).findings);
}

#[test]
fn a_malformed_rule_is_e662_and_is_not_evaluated() {
    let bad = [
        "kind: forbid-everything\nfrom: {tag: trip}\nto: {domain: software}\n",
        "kind: forbid-dependency\nto: {domain: software}\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\nto: {domain: software}\nvia: [telepathy]\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\nto: {domain: software}\nvia: []\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\nto: {domain: software}\nseverity: fatal\n",
        "kind: forbid-dependency\nfrom: {colour: red}\nto: {domain: software}\n",
        "kind: forbid-dependency\nfrom: {}\nto: {domain: software}\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\nto: {domain: firmware}\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\nto: {stereotype: 'Nonesuch'}\n",
        "kind: forbid-dependency\nfrom: {tag: trip}\nto: {stereotype: 'Safety where >='}\n",
    ];
    for r in bad {
        let els = model(&format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n"), r);
        let e = codes(&els, "E662");
        assert!(!e.is_empty(), "no E662 for {r:?}: {:?}", validate(&els).findings);
        assert!(codes(&els, "E660").is_empty(), "a malformed rule is not evaluated: {r:?}");
    }
}

#[test]
fn a_selector_that_chooses_nothing_is_w662() {
    let els = model(SENSE, "kind: forbid-dependency\nfrom: {tag: nothing-has-this}\nto: {domain: software}\n");
    let w = codes(&els, "W662");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].1.contains("from"), "{w:?}");
}

#[test]
fn selectors_and_together_package_tag_domain() {
    // tag trip AND domain hardware AND package Hw: both trip parts; only Sensor is not software-adjacent
    let r = "kind: forbid-dependency\nfrom: {package: Hw, tag: trip, domain: hardware}\nto: {domain: software}\n";
    let wired = format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n");
    assert_eq!(codes(&model(&wired, r), "E660").len(), 1);
    let none = "kind: forbid-dependency\nfrom: {package: Sw, tag: trip}\nto: {domain: software}\n";
    assert_eq!(codes(&model(&wired, none), "W662").len(), 1, "no Sw component has the trip tag");
}

#[test]
fn the_fields_are_unrecognised_on_other_elements() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "P.md", "---\ntype: PartDef\nname: P\nkind: independence\nfrom: {tag: x}\nto: {tag: y}\nvia: [connections]\n---\n\nP.\n");
    let els = walk_model(&r).unwrap();
    let w: Vec<_> = codes(&els, "W047");
    for f in ["kind", "from", "to", "via"] {
        assert!(w.iter().any(|(_, m)| m.contains(&format!("'{f}'"))), "{f}: {w:?}");
    }
}

// ── one wiring mistake is one finding, not one per contained component ─────────────────────────

/// `Sw::Bundle`: a software composite containing `Fw` and `Sup`, with an output port; `Top` gets a `bundle` usage.
fn bundled(connections: &str, rule: &str) -> Vec<RawElement> {
    model_with(
        connections,
        rule,
        &[("Sw/Bundle.md", "---\ntype: PartDef\nname: Bundle\ndomain: software\nfeatures:\n  - {name: fw, typedBy: Sw::Fw}\n  - {name: sup, typedBy: Sw::Sup}\n  - {name: o, type: Port, direction: out}\n---\n\nBundle.\n")],
        "  - {name: bundle, typedBy: Sw::Bundle}\n",
    )
}

const FED_BY_BUNDLE: &str = "  - {from: sensor.o, to: actuator.i}\n  - {from: bundle.o, to: actuator.cmd}\n";

#[test]
fn forbid_dependency_stops_at_the_first_forbidden_component() {
    let e = codes(&bundled(FED_BY_BUNDLE, FORBID_SW), "E660");
    assert_eq!(e.len(), 1, "Bundle, not also everything inside it: {e:?}");
    assert!(in_order(&e[0].1, &["Hw::Actuator", "Sw::Bundle"]) && !e[0].1.contains("Sw::Fw"), "{e:?}");
}

#[test]
fn a_dependency_through_another_source_is_reported_once_at_that_source() {
    // the actuator depends on the sensor, and the sensor on the supervisor: the finding is the sensor's
    let e = codes(&model(&format!("{SENSE}  - {{from: sup.o, to: sensor.cfg}}\n"), FORBID_SW), "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(in_order(&e[0].1, &["Hw::Sensor", "Sw::Sup"]) && !e[0].1.contains("Hw::Actuator"), "{e:?}");
}

#[test]
fn independence_reports_the_shared_component_not_what_it_contains() {
    // the bundle feeds the actuator: it is in both closures, and so are Fw, Sup and the Mcu Fw runs on
    let e = codes(&bundled(FED_BY_BUNDLE, INDEP), "E661");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].1.contains("Sw::Bundle is shared"), "the root cause: {e:?}");
}

#[test]
fn two_independent_causes_are_two_findings() {
    // the supervisor feeds the actuator, and the MCU the sensor: Sup and Mcu are shared and neither reaches the other
    let e = codes(&model(&format!("{SENSE}  - {{from: sup.o, to: actuator.cmd}}\n  - {{from: mcu.o, to: sensor.cfg}}\n"), INDEP), "E661");
    let names: Vec<_> = e.iter().map(|(_, m)| m.as_str()).collect();
    assert!(names.iter().any(|m| m.contains("Sw::Sup is shared")) && names.iter().any(|m| m.contains("Hw::Mcu is shared")), "{e:?}");
    assert_eq!(e.len(), 2, "{e:?}");
}

// ── a stereotype selector chooses only the components that apply it (REQ-TRS-ARCH-002, REQ-TRS-META-053) ──

#[test]
fn a_stereotype_selector_chooses_only_the_components_that_apply_it() {
    // `Gate` applies Marker and takes input from the supervisor; the unmarked actuator does too.
    let els = model_with(
        &format!("{SENSE}  - {{from: sup.o, to: gate.i}}\n  - {{from: sup.o, to: actuator.cmd}}\n"),
        "kind: forbid-dependency\nfrom: {stereotype: Marker}\nto: {domain: software}\n",
        &[
            ("Hw/Marker.md", "---\ntype: MetadataDef\nname: Marker\n---\n"),
            ("Hw/Gate.md", "---\ntype: PartDef\nname: Gate\ndomain: hardware\nmetadata:\n  - Hw::Marker\nfeatures:\n  - {name: i, type: Port, direction: in}\n---\n\nGate.\n"),
        ],
        "  - {name: gate, typedBy: Hw::Gate}\n",
    );
    let e = codes(&els, "E660");
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(in_order(&e[0].1, &["Hw::Gate", "Sw::Sup"]) && !e[0].1.contains("Hw::Actuator"), "{e:?}");
}
