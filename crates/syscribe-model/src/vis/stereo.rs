//! Diagram style driven by stereotypes (`REQ-TRS-META-040`..`-044`, GH #378).
//!
//! [`apply`] puts the `[diagram.stereotypes.<Def>]` table of `.syscribe.toml` on a built graph:
//!
//! * every node whose element applies a configured stereotype (or a specialisation of it,
//!   `REQ-TRS-META-030`) under its optional tagged-value condition gets the composed presentation in
//!   [`Node::stereo`], which [`super::style::node_style`] folds into the node's style for every writer;
//! * every connection whose entry, or `ConnectionDef`, applies one gets it in
//!   [`DiagramGraph::edge_stereo`], folded in by [`super::style::edge_style_in`] (`-042`);
//! * on an IBD with `groupBy: stereotype`, the parts that apply a stereotype marked `frame = true` are
//!   gathered into one swimlane per such stereotype (`-041`);
//! * the graph gets a legend of the stereotypes that actually styled something (`-043`).
//!
//! Without the table nothing changes.

use super::flow::{DiagramConfig, StereotypeStyle};
use super::ir::{DiagramGraph, LegendItem, Node, NodeKind, StereoStyle};
use crate::element::RawElement;
use crate::resolver::Resolver;
use crate::tagged_values::{self as tv, EffectiveApp};

/// The configured styles that apply through one of `apps` (the applications of `carrier`), with that
/// application's definition: `(style key, style, application definition)`, weakest first.
fn matching<'a>(
    elements: &[RawElement],
    resolver: &Resolver,
    carrier: &RawElement,
    apps: &[EffectiveApp],
    cfg: &'a DiagramConfig,
) -> Vec<(&'a String, &'a StereotypeStyle, String)> {
    let mut out: Vec<(&String, &StereotypeStyle, String)> = Vec::new();
    for app in apps {
        for (key, style) in &cfg.stereotypes {
            if !tv::names_in_lineage(key, &app.lineage) || out.iter().any(|(k, _, d)| *k == key && *d == app.def) {
                continue;
            }
            if let Some(w) = &style.when {
                if !tv::application_holds(elements, resolver, carrier, app, w) {
                    continue;
                }
            }
            out.push((key, style, app.def.clone()));
        }
    }
    // Weakest first, so a stronger style overwrites a property both set.
    out.sort_by(|a, b| (a.1.priority, a.0).cmp(&(b.1.priority, b.0)));
    out
}

/// Fold styles weakest to strongest, property by property.
fn compose(hits: &[(&String, &StereotypeStyle, String)]) -> StereoStyle {
    let mut c = StereoStyle::default();
    for (_, s, _) in hits {
        c.fill = s.fill.clone().or(c.fill);
        c.stroke = s.stroke.clone().or(c.stroke);
        c.dashed = s.dashed.or(c.dashed);
        c.width = s.width.or(c.width);
    }
    c
}

fn note_used(used: &mut Vec<(i64, String)>, hits: &[(&String, &StereotypeStyle, String)]) {
    for (key, style, _) in hits {
        if style.is_visual() && !used.iter().any(|(_, k)| k == *key) {
            used.push((style.priority, (*key).clone()));
        }
    }
}

pub fn apply(graph: &mut DiagramGraph, elements: &[RawElement], resolver: &Resolver, cfg: &DiagramConfig) {
    if cfg.stereotypes.is_empty() {
        return;
    }
    // (priority, key) of every style that styled something, for the legend
    let mut used: Vec<(i64, String)> = Vec::new();
    // (node id, strongest frame stereotype key) for `groupBy: stereotype`
    let mut framed: Vec<(String, String, i64)> = Vec::new();

    let node_defs = std::mem::take(&mut graph.node_defs);
    for node in graph.nodes.iter_mut().filter(|n| n.resolved) {
        // The node's own element (a model element, or none for an inline usage) and, for a part usage, its
        // definition: a usage takes the stereotypes of what it is a usage of.
        let own = resolver.resolve_ref(elements, &node.element_ref);
        let def = node_defs.get(&node.id).and_then(|d| resolver.resolve_ref(elements, d));
        let Some(elem) = own.or(def) else { continue };
        let mut apps = tv::effective_applications(elements, resolver, elem);
        if let (Some(d), Some(_)) = (def, own) {
            apps.extend(tv::effective_applications(elements, resolver, d));
        }
        let hits = matching(elements, resolver, elem, &apps, cfg);
        if hits.is_empty() {
            continue;
        }
        note_used(&mut used, &hits);
        for (_, style, app_def) in &hits {
            if style.banner == Some(false) {
                // the banner is the definition's name (or the reference's last segment)
                let short = app_def.rsplit("::").next().unwrap_or(app_def);
                node.banners.retain(|b| b != short);
            }
        }
        if graph.group_by_stereotype {
            if let Some((key, style, _)) = hits.iter().rev().find(|(_, s, _)| s.frame == Some(true)) {
                framed.push((node.id.clone(), (*key).clone(), style.priority));
            }
        }
        let composed = compose(&hits);
        if composed != StereoStyle::default() {
            node.stereo = Some(composed);
        }
    }

    // Connections: the entry's own applications and its ConnectionDef's.
    let edge_meta = std::mem::take(&mut graph.edge_meta);
    for (edge_id, meta) in &edge_meta {
        let Some(edge) = graph.edges.iter().find(|e| &e.id == edge_id) else { continue };
        let Some(owner) = edge.element_ref.as_deref().and_then(|r| resolver.resolve_ref(elements, r)) else { continue };
        let apps = tv::applications_from(elements, resolver, &Some(meta.clone()));
        let hits = matching(elements, resolver, owner, &apps, cfg);
        let composed = compose(&hits);
        if composed != StereoStyle::default() {
            note_used(&mut used, &hits);
            graph.edge_stereo.insert(edge_id.clone(), composed);
        }
    }
    graph.edge_meta = edge_meta;
    graph.node_defs = node_defs;

    frame_by_stereotype(graph, &framed, cfg);

    // Strongest first: the order a reader resolves a clash in.
    used.sort_by(|a, b| (b.0, &b.1).cmp(&(a.0, &a.1)));
    graph.legend = used
        .into_iter()
        .map(|(_, key)| {
            let s = &cfg.stereotypes[&key];
            LegendItem {
                name: key.rsplit("::").next().unwrap_or(&key).to_string(),
                fill: s.fill.clone(),
                stroke: s.stroke.clone(),
                dashed: s.dashed.unwrap_or(false),
                width: s.width,
            }
        })
        .collect();
}

/// One swimlane per frame stereotype, holding the top-level parts whose strongest frame stereotype it is;
/// strongest stereotype first. A part with none stays in the boundary.
fn frame_by_stereotype(graph: &mut DiagramGraph, framed: &[(String, String, i64)], cfg: &DiagramConfig) {
    if framed.is_empty() {
        return;
    }
    let Some(boundary) = graph.nodes.iter().find(|n| n.kind == NodeKind::Boundary).map(|n| n.id.clone()) else { return };
    let mut keys: Vec<(&String, i64)> = Vec::new();
    for (_, key, prio) in framed {
        if !keys.iter().any(|(k, _)| *k == key) {
            keys.push((key, *prio));
        }
    }
    keys.sort_by(|a, b| (b.1, b.0).cmp(&(a.1, a.0)));
    for (key, _) in keys {
        let top: Vec<&String> = framed
            .iter()
            .filter(|(id, k, _)| k == key && graph.node(id).is_some_and(|n| n.parent.as_deref() == Some(boundary.as_str())))
            .map(|(id, _, _)| id)
            .collect();
        if top.is_empty() {
            continue;
        }
        let name = key.rsplit("::").next().unwrap_or(key).to_string();
        let mut id = format!("s-stereotype-{}", name.to_lowercase());
        while graph.node(&id).is_some() {
            id.push('x');
        }
        let ids: Vec<String> = top.into_iter().cloned().collect();
        graph.nodes.push(Node {
            id: id.clone(),
            element_ref: graph.subject.clone().unwrap_or_default(),
            resolved: true,
            element_type: None,
            kind: NodeKind::Swimlane,
            label: name,
            stereotype: Some("stereotype".to_string()),
            parent: Some(boundary.clone()),
            direction: None,
            side: None,
            lines: Vec::new(),
            is_abstract: false,
            pin: None,
            banners: Vec::new(),
            stereo: None,
            feature: None,
            mark: None,
            flow: None,
            faded: false,
        });
        for n in graph.nodes.iter_mut().filter(|n| ids.contains(&n.id)) {
            n.parent = Some(id.clone());
        }
        let _ = cfg;
    }
    graph.parents_first();
}
