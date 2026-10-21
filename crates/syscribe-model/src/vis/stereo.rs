//! Diagram style driven by stereotypes (`REQ-TRS-META-040`..`-044`, GH #378).
//!
//! [`apply`] puts the `[diagram.stereotypes.<Def>]` table of `.syscribe.toml` on a built graph: every
//! node whose element applies a configured stereotype (or a specialisation of it, `REQ-TRS-META-030`)
//! under its optional tagged-value condition gets the composed presentation in [`Node::stereo`], which
//! [`super::style::node_style`] folds into the node's style for every writer, and the graph gets a
//! legend of the stereotypes that actually styled something. Without the table nothing changes.

use super::flow::{DiagramConfig, StereotypeStyle};
use super::ir::{DiagramGraph, LegendItem, StereoStyle};
use crate::element::RawElement;
use crate::resolver::Resolver;
use crate::tagged_values as tv;

fn names(def: &str, lineage: &[String]) -> bool {
    lineage.iter().any(|q| q == def || (!def.contains("::") && q.rsplit("::").next() == Some(def)))
}

/// The configured styles that apply to `elem` through one of its applications, with that
/// application's definition: `(style key, style, application definition)`.
fn matching<'a>(
    elements: &[RawElement],
    resolver: &Resolver,
    elem: &RawElement,
    cfg: &'a DiagramConfig,
) -> Vec<(&'a String, &'a StereotypeStyle, String)> {
    let mut out: Vec<(&String, &StereotypeStyle, String)> = Vec::new();
    for app in tv::effective_applications(elements, resolver, elem) {
        for (key, style) in &cfg.stereotypes {
            if !names(key, &app.lineage) || out.iter().any(|(k, _, d)| *k == key && *d == app.def) {
                continue;
            }
            if let Some(w) = &style.when {
                let Ok(expr) = tv::parse_constraint(w) else { continue };
                let specs = tv::find_metadata_def(elements, resolver, &app.def)
                    .map(|d| tv::merged_specs(&tv::lineage(elements, resolver, d)))
                    .unwrap_or_default();
                let scope = tv::Scope { elements, resolver, carrier: elem };
                if tv::evaluate(&expr, &specs, &app.values, Some(&scope)) != tv::Verdict::Holds {
                    continue;
                }
            }
            out.push((key, style, app.def.clone()));
        }
    }
    out
}

pub fn apply(graph: &mut DiagramGraph, elements: &[RawElement], resolver: &Resolver, cfg: &DiagramConfig) {
    if cfg.stereotypes.is_empty() {
        return;
    }
    // (priority, key) of every style that styled a node, for the legend
    let mut used: Vec<(i64, String)> = Vec::new();
    for node in graph.nodes.iter_mut().filter(|n| n.resolved) {
        let Some(elem) = resolver.resolve_ref(elements, &node.element_ref) else { continue };
        let mut hits = matching(elements, resolver, elem, cfg);
        if hits.is_empty() {
            continue;
        }
        // Weakest first, so a stronger style overwrites a property both set.
        hits.sort_by(|a, b| (a.1.priority, a.0).cmp(&(b.1.priority, b.0)));
        let mut composed = StereoStyle::default();
        for (key, style, app_def) in &hits {
            if style.is_visual() && !used.iter().any(|(_, k)| k == *key) {
                used.push((style.priority, (*key).clone()));
            }
            composed.fill = style.fill.clone().or(composed.fill);
            composed.stroke = style.stroke.clone().or(composed.stroke);
            composed.dashed = style.dashed.or(composed.dashed);
            composed.width = style.width.or(composed.width);
            if style.banner == Some(false) {
                // the banner is the definition's name (or the reference's last segment)
                let short = app_def.rsplit("::").next().unwrap_or(app_def);
                node.banners.retain(|b| b != short);
            }
        }
        if composed != StereoStyle::default() {
            node.stereo = Some(composed);
        }
    }
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
