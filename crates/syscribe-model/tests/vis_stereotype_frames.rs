//! TC-TRS-META-041, -042 / REQ-TRS-META-041, -042 (GH #378): a stereotype as an IBD frame criterion,
//! and stereotypes on connections styling the edge.
//!
//! Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::vis::style::edge_style_in;
use syscribe_model::vis::{build_graph_in, render_svg, DiagramGraph, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-stereoframes-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// An inverter IBD: `Bus` → `Bridge` (a connection that applies `Safety`), `Ctl` → `Bridge` (a plain
/// connection), and `Sensor` → `Ctl` (a connection typed by a `ConnectionDef` that applies `Safety`).
/// `Sensor` and `Ctl` apply `External`; `Bus` applies nothing. `group_by` is the diagram's `groupBy:`.
fn model(toml: &str, group_by: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Inv/_index.md", "---\ntype: Package\nname: Inv\n---\n");
    write(&root, "Inv/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n---\n");
    write(&root, "Inv/External.md", "---\ntype: MetadataDef\nname: External\n---\n");
    write(&root, "Inv/Link.md", "---\ntype: ConnectionDef\nname: Link\nmetadata:\n  - type: Inv::Safety\n    integrity: 1\n---\n");
    write(&root, "Inv/Port.md", "---\ntype: PortDef\nname: Port\n---\n");
    let part = |n: &str, ports: &str, meta: &str| {
        write(&root, &format!("Inv/{n}.md"), &format!("---\ntype: PartDef\nname: {n}\n{meta}features:\n{ports}---\n\n{n}.\n"))
    };
    let out = "  - name: o\n    typedBy: Inv::Port\n    direction: out\n";
    let inn = "  - name: i\n    typedBy: Inv::Port\n    direction: in\n";
    part("Bus", out, "");
    part("Bridge", &format!("{inn}  - name: j\n    typedBy: Inv::Port\n    direction: in\n"), "");
    part("Ctl", &format!("{out}{inn}"), "metadata:\n  - Inv::External\n");
    part("Sensor", out, "metadata:\n  - Inv::External\n");
    write(
        &root,
        "Inv/Inverter.md",
        "---\ntype: PartDef\nname: Inverter\nfeatures:\n  - name: bus\n    typedBy: Inv::Bus\n  - name: bridge\n    typedBy: Inv::Bridge\n  - name: ctl\n    typedBy: Inv::Ctl\n  - name: sensor\n    typedBy: Inv::Sensor\nconnections:\n  - from: bus.o\n    to: bridge.i\n    metadata:\n      - type: Inv::Safety\n        integrity: 3\n  - from: ctl.o\n    to: bridge.j\n  - from: sensor.o\n    to: ctl.i\n    typedBy: Inv::Link\n---\n",
    );
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(&root, "Diagrams/InvIBD.md", &format!("---\ntype: Diagram\nname: InvIBD\ndiagramKind: IBD\nsubject: Inv::Inverter\n{group_by}---\n\nIBD.\n"));
    if !toml.is_empty() {
        write(&root, ".syscribe.toml", toml);
    }
    root
}

fn graph(root: &Path) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::InvIBD").unwrap();
    let (g, issues) = build_graph_in(d, &elements, &resolver, root).unwrap();
    assert!(g.nodes.len() > 4, "the fixture IBD has parts; issues: {issues:?}");
    g
}

fn no_links(_: &str) -> Option<String> {
    None
}

const FRAME: &str = "[diagram.stereotypes.External]\nframe = true\nstroke = \"#555555\"\n";

// ── REQ-TRS-META-041 ───────────────────────────────────────────────────────

fn lane<'a>(g: &'a DiagramGraph, label: &str) -> Option<&'a syscribe_model::vis::Node> {
    g.nodes.iter().find(|n| n.kind == NodeKind::Swimlane && n.label == label)
}

fn part<'a>(g: &'a DiagramGraph, name: &str) -> &'a syscribe_model::vis::Node {
    g.nodes.iter().find(|n| n.kind != NodeKind::Port && n.kind != NodeKind::Swimlane && n.label.starts_with(&format!("{name} "))).unwrap_or_else(|| panic!("part {name}: {:?}", g.nodes.iter().map(|n| (&n.id, &n.label)).collect::<Vec<_>>()))
}

#[test]
fn group_by_stereotype_frames_the_parts_that_apply_a_frame_stereotype() {
    let g = graph(&model(FRAME, "groupBy: stereotype\n"));
    let l = lane(&g, "External").expect("an External lane");
    assert_eq!(part(&g, "sensor").parent.as_deref(), Some(l.id.as_str()));
    assert_eq!(part(&g, "ctl").parent.as_deref(), Some(l.id.as_str()));
    assert_ne!(part(&g, "bus").parent.as_deref(), Some(l.id.as_str()), "Bus applies nothing");
    assert_ne!(part(&g, "bridge").parent.as_deref(), Some(l.id.as_str()));
}

#[test]
fn a_stereotype_without_frame_true_makes_no_lane() {
    let g = graph(&model("[diagram.stereotypes.External]\nstroke = \"#555555\"\n", "groupBy: stereotype\n"));
    assert!(lane(&g, "External").is_none());
}

#[test]
fn without_group_by_stereotype_nothing_is_framed() {
    assert!(lane(&graph(&model(FRAME, "")), "External").is_none());
}

#[test]
fn a_frame_stereotype_also_frames_its_specialisations() {
    let root = model(FRAME, "groupBy: stereotype\n");
    write(&root, "Inv/Remote.md", "---\ntype: MetadataDef\nname: Remote\nsupertype: Inv::External\n---\n");
    write(&root, "Inv/Sensor.md", "---\ntype: PartDef\nname: Sensor\nmetadata:\n  - Inv::Remote\nfeatures:\n  - name: o\n    typedBy: Inv::Port\n    direction: out\n---\n\nSensor.\n");
    let g = graph(&root);
    assert_eq!(part(&g, "sensor").parent.as_deref(), lane(&g, "External").map(|l| l.id.as_str()));
}

// ── REQ-TRS-META-042 ───────────────────────────────────────────────────────

fn edge_stroke(g: &DiagramGraph, from_suffix: &str) -> String {
    let e = g.edges.iter().find(|e| e.source.contains(from_suffix) || e.id.contains(from_suffix)).unwrap_or_else(|| panic!("edge {from_suffix}: {:?}", g.edges.iter().map(|e| (&e.id, &e.source)).collect::<Vec<_>>()));
    edge_style_in(e, g).stroke
}

const SAFE_EDGE: &str = "[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\nwidth = 3\ndash = \"6,3\"\n";

#[test]
fn a_stereotype_on_a_connection_entry_styles_the_edge() {
    let g = graph(&model(SAFE_EDGE, ""));
    let e = g.edges.iter().find(|e| e.source.contains("bus")).expect("bus edge");
    let st = edge_style_in(e, &g);
    assert_eq!((st.stroke.as_str(), st.width, st.dash.as_deref()), ("#d00000", 3.0, Some("6,3")));
}

#[test]
fn a_stereotype_on_the_connection_def_styles_the_edge_too() {
    let g = graph(&model(SAFE_EDGE, ""));
    assert_eq!(edge_stroke(&g, "sensor"), "#d00000", "the edge typed by Link, which applies Safety");
}

#[test]
fn an_unstereotyped_edge_keeps_its_look() {
    let g = graph(&model(SAFE_EDGE, ""));
    assert_ne!(edge_stroke(&g, "ctl"), "#d00000");
}

#[test]
fn a_tagged_value_condition_applies_to_the_connection_application() {
    // The entry applies integrity 3, the def integrity 1: only the former satisfies `integrity >= 2`.
    let g = graph(&model("[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\nwhen = \"integrity >= 2\"\n", ""));
    assert_eq!(edge_stroke(&g, "bus"), "#d00000");
    assert_ne!(edge_stroke(&g, "sensor"), "#d00000");
}

#[test]
fn an_edge_style_reaches_the_legend_and_the_svg() {
    let g = graph(&model(SAFE_EDGE, ""));
    assert!(g.legend.iter().any(|l| l.name == "Safety"), "{:?}", g.legend);
    assert!(render_svg(&g, &no_links).unwrap().contains("#d00000"));
}

#[test]
fn removing_the_style_restores_the_output() {
    let base = graph(&model("", ""));
    let off = graph(&model("[ids]\nmax_digits = 8\n", ""));
    assert_eq!(serde_json::to_string(&base).unwrap(), serde_json::to_string(&off).unwrap());
}
