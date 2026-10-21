//! TC-TRS-META-040..044 / REQ-TRS-META-040..044 (GH #378): diagram style driven by stereotypes.
//!
//! `[diagram.stereotypes.<Def>]` in `.syscribe.toml` maps a stereotype (optionally under a
//! tagged-value condition) to presentation. Written before the implementation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::vis::style::node_style;
use syscribe_model::vis::{build_graph_in, render_dot, render_mermaid, render_svg, DiagramGraph, Node};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-stereo-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// `P1` Safety(3), `P2` Safety(1), `P3` Optional, `P4` nothing, `P5` Safety(2)+Optional,
/// `P6` ASIL_D (a specialisation of Safety, integrity 4). `toml` is the `.syscribe.toml`.
fn model(toml: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "B/_index.md", "---\ntype: Package\nname: B\n---\n");
    write(&root, "B/Safety.md", "---\ntype: MetadataDef\nname: Safety\nfeatures:\n  - name: integrity\n    typedBy: ScalarValues::Integer\n---\n");
    write(&root, "B/ASIL_D.md", "---\ntype: MetadataDef\nname: ASIL_D\nsupertype: B::Safety\n---\n");
    write(&root, "B/Optional.md", "---\ntype: MetadataDef\nname: Optional\n---\n");
    let part = |n: &str, meta: &str| write(&root, &format!("B/{n}.md"), &format!("---\ntype: PartDef\nname: {n}\n{meta}---\n\n{n}.\n"));
    part("P1", "metadata:\n  - type: B::Safety\n    integrity: 3\n");
    part("P2", "metadata:\n  - type: B::Safety\n    integrity: 1\n");
    part("P3", "metadata:\n  - B::Optional\n");
    part("P4", "");
    part("P5", "metadata:\n  - type: B::Safety\n    integrity: 2\n  - B::Optional\n");
    part("P6", "metadata:\n  - type: B::ASIL_D\n    integrity: 4\n");
    write(
        &root,
        "B/D.md",
        "---\ntype: Diagram\nname: D\ndiagramKind: BDD\nsubject: B::P1\nshapes:\n  - {id: s1, ref: B::P1}\n  - {id: s2, ref: B::P2}\n  - {id: s3, ref: B::P3}\n  - {id: s4, ref: B::P4}\n  - {id: s5, ref: B::P5}\n  - {id: s6, ref: B::P6}\n---\n\nD.\n",
    );
    if !toml.is_empty() {
        write(&root, ".syscribe.toml", toml);
    }
    root
}

fn graph(root: &Path) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "B::D").unwrap();
    let (g, issues) = build_graph_in(d, &elements, &resolver, root).unwrap();
    assert!(!g.nodes.is_empty(), "the fixture diagram has no nodes; issues: {issues:?}");
    g
}

fn node<'a>(g: &'a DiagramGraph, id: &str) -> &'a Node {
    g.nodes.iter().find(|n| n.id == id).unwrap_or_else(|| panic!("node {id}: {:?}", g.nodes.iter().map(|n| &n.id).collect::<Vec<_>>()))
}

fn no_links(_: &str) -> Option<String> {
    None
}

const RED: &str = "[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\nfill = \"#ffe5e5\"\nwidth = 3\n";

// ── REQ-TRS-META-040 ───────────────────────────────────────────────────────

#[test]
fn a_configured_stereotype_restyles_the_elements_that_apply_it() {
    let g = graph(&model(RED));
    for id in ["s1", "s2", "s5"] {
        let st = node_style(node(&g, id));
        assert_eq!((st.stroke.as_str(), st.fill.as_str(), st.stroke_width), ("#d00000", "#ffe5e5", Some(3.0)), "{id}");
    }
    let plain = node_style(node(&g, "s4"));
    assert_ne!(plain.stroke, "#d00000", "an element without the stereotype keeps its kind's colours");
}

#[test]
fn removing_the_style_restores_the_output_byte_for_byte() {
    let baseline = graph(&model(""));
    let unrelated = graph(&model("[ids]\nmax_digits = 8\n"));
    assert_eq!(serde_json::to_string(&baseline).unwrap(), serde_json::to_string(&unrelated).unwrap());
    assert_eq!(render_svg(&baseline, &no_links).unwrap(), render_svg(&unrelated, &no_links).unwrap());
    let styled = graph(&model(RED));
    assert_ne!(render_svg(&baseline, &no_links).unwrap(), render_svg(&styled, &no_links).unwrap());
}

#[test]
fn a_tagged_value_condition_selects_the_style_only_when_true() {
    let g = graph(&model("[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\nwhen = \"integrity >= 2\"\n"));
    assert_eq!(node_style(node(&g, "s1")).stroke, "#d00000", "integrity 3");
    assert_eq!(node_style(node(&g, "s5")).stroke, "#d00000", "integrity 2");
    assert_ne!(node_style(node(&g, "s2")).stroke, "#d00000", "integrity 1");
}

#[test]
fn several_stereotypes_compose_by_priority_per_property() {
    let toml = "[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\nfill = \"#ffe5e5\"\n[diagram.stereotypes.Optional]\nstroke = \"#0050d0\"\npriority = 5\n";
    let g = graph(&model(toml));
    let s5 = node_style(node(&g, "s5"));
    assert_eq!(s5.stroke, "#0050d0", "the higher priority wins the property both set");
    assert_eq!(s5.fill, "#ffe5e5", "a property only one sets is kept");
    let toml2 = "[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\npriority = 5\n[diagram.stereotypes.Optional]\nstroke = \"#0050d0\"\n";
    assert_eq!(node_style(node(&graph(&model(toml2)), "s5")).stroke, "#d00000");
}

#[test]
fn equal_priorities_are_decided_by_definition_name() {
    let toml = "[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\n[diagram.stereotypes.Optional]\nstroke = \"#0050d0\"\n";
    let a = node_style(node(&graph(&model(toml)), "s5")).stroke;
    let b = node_style(node(&graph(&model(toml)), "s5")).stroke;
    assert_eq!(a, b, "deterministic");
    assert_eq!(a, "#d00000", "the later name (Safety) wins a tie");
}

#[test]
fn a_dashed_style_dashes_the_outline() {
    let g = graph(&model("[diagram.stereotypes.Optional]\ndash = \"6,3\"\n"));
    assert!(node_style(node(&g, "s3")).dashed);
    assert!(!node_style(node(&g, "s4")).dashed);
}

#[test]
fn banner_false_hides_that_stereotypes_banner_only() {
    let base = graph(&model(""));
    assert_eq!(node(&base, "s5").banners, ["Safety", "Optional"]);
    let g = graph(&model("[diagram.stereotypes.Safety]\nbanner = false\n"));
    assert_eq!(node(&g, "s5").banners, ["Optional"]);
    assert_eq!(node(&g, "s1").banners, Vec::<String>::new());
}

#[test]
fn a_malformed_entry_is_ignored_and_reported() {
    let cfg = syscribe_model::vis::flow::DiagramConfig::from_toml_str("[diagram.stereotypes.Safety]\nstroke = \"not a colour!\"\nwhen = \"integrity >=\"\nwidth = -1\nbogus = 1\n");
    assert!(cfg.problems.iter().any(|p| p.contains("stroke")), "{:?}", cfg.problems);
    assert!(cfg.problems.iter().any(|p| p.contains("when")), "{:?}", cfg.problems);
    assert!(cfg.problems.iter().any(|p| p.contains("width")), "{:?}", cfg.problems);
    assert!(cfg.problems.iter().any(|p| p.contains("bogus")), "{:?}", cfg.problems);
}

// ── specialisation (REQ-TRS-META-030) ──────────────────────────────────────

#[test]
fn a_style_on_a_supertype_applies_to_its_specialisations() {
    let g = graph(&model(RED));
    assert_eq!(node_style(node(&g, "s6")).stroke, "#d00000");
    let only_sub = graph(&model("[diagram.stereotypes.ASIL_D]\nstroke = \"#800000\"\n"));
    assert_eq!(node_style(node(&only_sub, "s6")).stroke, "#800000");
    assert_ne!(node_style(node(&only_sub, "s1")).stroke, "#800000", "a plain Safety is not an ASIL_D");
}

// ── REQ-TRS-META-043 ───────────────────────────────────────────────────────

fn legend_names(g: &DiagramGraph) -> Vec<String> {
    g.legend.iter().map(|l| l.name.clone()).collect()
}

#[test]
fn the_legend_lists_exactly_the_stereotypes_that_styled_something() {
    let toml = "[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\n[diagram.stereotypes.Optional]\ndash = \"6,3\"\n[diagram.stereotypes.External]\nstroke = \"#111111\"\n";
    let g = graph(&model(toml));
    assert_eq!(legend_names(&g), ["Safety", "Optional"], "External styled nothing in this diagram");
    // A condition that never holds draws nothing and so is not in the legend.
    let never = graph(&model("[diagram.stereotypes.Safety]\nstroke = \"#d00000\"\nwhen = \"integrity > 99\"\n"));
    assert!(never.legend.is_empty());
}

#[test]
fn no_style_means_no_legend() {
    let g = graph(&model(""));
    assert!(g.legend.is_empty());
    assert!(!render_svg(&g, &no_links).unwrap().contains("class=\"legend\""));
}

#[test]
fn the_svg_draws_the_legend_with_the_stereotype_names() {
    let svg = render_svg(&graph(&model(RED)), &no_links).unwrap();
    assert!(svg.contains("class=\"legend\"") && svg.contains("«Safety»"), "{svg}");
}

// ── REQ-TRS-META-044 ───────────────────────────────────────────────────────

#[test]
fn every_writer_carries_the_same_resolved_style() {
    let g = graph(&model(RED));
    let svg = render_svg(&g, &no_links).unwrap();
    assert!(svg.contains("#d00000") && svg.contains("#ffe5e5"), "svg");
    assert!(render_dot(&g).contains("#d00000"), "dot");
    assert!(render_mermaid(&g, &no_links).unwrap().contains("#d00000"), "mermaid");
    let sprotty = syscribe_model::vis::sprotty::to_sgraph_json(&g).to_string();
    assert!(sprotty.contains("#d00000"), "sprotty");
}
