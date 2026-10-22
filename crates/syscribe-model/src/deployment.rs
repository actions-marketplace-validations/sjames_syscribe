//! Deployment structure (`REQ-TRS-DEPL-001`..`-003`, GH #370): `deploymentKind:` on parts, nesting order, a
//! component deployed once unless `replicated`, and the integrity of a host against what it hosts.
//!
//! Opt-in: a part that declares no `deploymentKind` is not a node, and nothing is checked about it.

use std::collections::BTreeSet;

use crate::arch_rules::{build_graph, Via};
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// `(is_error, code, file, message)`.
pub type Finding = (bool, &'static str, String, String);

/// Outermost first.
const KINDS: [&str; 6] = ["soc", "core", "partition", "container", "process", "thread"];

fn is_part(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef | ElementType::Part))
}

fn rank(e: &RawElement) -> Option<usize> {
    let k = e.frontmatter.deployment_kind.as_deref()?;
    KINDS.iter().position(|x| *x == k)
}

/// `QM` < `A` < `B` < `C` < `D`.
fn asil(s: &str) -> Option<u8> {
    match s.trim().to_ascii_uppercase().as_str() {
        "QM" => Some(0),
        "A" => Some(1),
        "B" => Some(2),
        "C" => Some(3),
        "D" => Some(4),
        _ => None,
    }
}

pub fn check(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for e in elements.iter().filter(|e| is_part(e)) {
        if let Some(k) = &e.frontmatter.deployment_kind {
            if !KINDS.contains(&k.as_str()) {
                out.push((true, "E730", e.file_path.clone(), format!("{}: deploymentKind '{k}' is not one of {}", e.qualified_name, KINDS.join(", "))));
            }
        }
    }
    if !elements.iter().any(|e| is_part(e) && rank(e).is_some()) {
        return out;
    }
    let by_name = |q: &str| elements.iter().find(|e| e.qualified_name == q);

    // nesting: a node contains only nodes of a later kind
    let contain: BTreeSet<Via> = [Via::TypedBy].into();
    let (gc, _) = build_graph(elements, resolver, &contain);
    for (x, deps) in gc.deps.iter().enumerate() {
        let Some(ex) = by_name(&gc.names[x]) else { continue };
        let Some(rx) = rank(ex) else { continue };
        for &y in deps.keys() {
            let Some(ey) = by_name(&gc.names[y]) else { continue };
            if let Some(ry) = rank(ey) {
                if ry <= rx {
                    out.push((
                        true,
                        "E731",
                        ex.file_path.clone(),
                        format!("{} ({}) contains {} ({}), which is not a later kind ({})", ex.qualified_name, KINDS[rx], ey.qualified_name, KINDS[ry], KINDS.join(" > ")),
                    ));
                }
            }
        }
    }

    // deployment: what each component is allocated to
    let alloc: BTreeSet<Via> = [Via::Allocations].into();
    let (ga, _) = build_graph(elements, resolver, &alloc);
    for (x, deps) in ga.deps.iter().enumerate() {
        let Some(comp) = by_name(&ga.names[x]) else { continue };
        let nodes: Vec<&RawElement> = deps.keys().filter_map(|&y| by_name(&ga.names[y])).filter(|n| rank(n).is_some()).collect();
        if nodes.len() > 1 && comp.frontmatter.replicated != Some(true) {
            let names = nodes.iter().map(|n| n.qualified_name.as_str()).collect::<Vec<_>>().join(", ");
            out.push((true, "E732", comp.file_path.clone(), format!("{} is allocated to {} deployment nodes ({names}); mark it `replicated: true` if that is intended", comp.qualified_name, nodes.len())));
        }
        for n in &nodes {
            let (host, guest) = (&n.frontmatter, &comp.frontmatter);
            let asil_pair = host.asil_level.as_deref().and_then(asil).zip(guest.asil_level.as_deref().and_then(asil));
            let sil_pair = host.sil_level.zip(guest.sil_level);
            let lower = match (asil_pair, sil_pair) {
                (Some((h, g)), _) if g > h => Some(format!("ASIL {} < ASIL {}", host.asil_level.as_deref().unwrap_or(""), guest.asil_level.as_deref().unwrap_or(""))),
                (_, Some((h, g))) if g > h => Some(format!("SIL {h} < SIL {g}")),
                _ => None,
            };
            if let Some(why) = lower {
                out.push((true, "E733", comp.file_path.clone(), format!("{} is allocated to {}, which has a lower integrity level ({why})", comp.qualified_name, n.qualified_name)));
            }
        }
    }
    out
}
