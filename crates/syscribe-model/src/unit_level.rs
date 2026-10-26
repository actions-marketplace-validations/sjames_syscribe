//! Software component and unit levels (`REQ-TRS-UNIT-001`..`-002`, GH #371): `level: component | unit` on a
//! software part, and for a unit the downward trace to code and tests and the upward trace to a requirement
//! and a component.
//!
//! Opt-in: a part that declares no `level` is not checked, and neither is a `draft` unit.

use std::collections::BTreeSet;

use crate::arch_rules::{build_graph, Via};
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// `(is_error, code, file, message)`.
pub type Finding = (bool, &'static str, String, String);

const LEVELS: [&str; 2] = ["component", "unit"];

fn is_part(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef | ElementType::Part))
}

/// A local path entry that names no `#symbol`. Remote URIs and package references are not judged.
fn lacks_symbol(entry: &str) -> bool {
    !entry.contains('#') && !crate::config::is_external_source_ref(entry)
}

pub fn check(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for e in elements.iter().filter(|e| is_part(e)) {
        if let Some(l) = &e.frontmatter.level {
            if !LEVELS.contains(&l.as_str()) {
                out.push((true, "E740", e.file_path.clone(), format!("{}: level '{l}' is not one of {}", e.qualified_name, LEVELS.join(", "))));
            }
        }
    }
    let units: Vec<&RawElement> = elements
        .iter()
        .filter(|e| is_part(e) && e.frontmatter.level.as_deref() == Some("unit") && e.frontmatter.status.as_deref() != Some("draft"))
        .collect();
    if units.is_empty() {
        return out;
    }

    // components that contain each unit, through `features:`
    let contain: BTreeSet<Via> = [Via::TypedBy].into();
    let (g, _) = build_graph(elements, resolver, &contain);
    let in_component = |unit: &RawElement| {
        g.deps.iter().enumerate().any(|(x, deps)| {
            let Some(holder) = elements.iter().find(|e| e.qualified_name == g.names[x]) else { return false };
            holder.frontmatter.level.as_deref() == Some("component")
                && deps.keys().any(|&y| g.names[y] == unit.qualified_name)
        })
    };

    // L1 tests: (verified qualified names) of every L1 TestCase
    let l1: Vec<BTreeSet<String>> = elements
        .iter()
        .filter(|t| t.frontmatter.element_type == Some(ElementType::TestCase) && t.frontmatter.test_level.as_deref() == Some("L1"))
        .map(|t| {
            t.frontmatter
                .verifies
                .iter()
                .flatten()
                .filter_map(|r| resolver.resolve_ref(elements, r))
                .map(|e| e.qualified_name.clone())
                .collect()
        })
        .collect();

    for u in units {
        let q = &u.qualified_name;
        let fm = &u.frontmatter;
        match fm.implemented_by.as_deref() {
            None | Some([]) => out.push((true, "E741", u.file_path.clone(), format!("{q} is a unit but has no `implementedBy:` code link"))),
            Some(entries) => {
                for entry in entries.iter().filter(|e| lacks_symbol(e)) {
                    out.push((false, "W740", u.file_path.clone(), format!("{q}: implementedBy '{entry}' names no symbol (use `path#symbol`)")));
                }
            }
        }
        let satisfied: BTreeSet<String> = fm
            .satisfies
            .iter()
            .flatten()
            .filter_map(|r| resolver.resolve_ref(elements, r))
            .map(|e| e.qualified_name.clone())
            .collect();
        if satisfied.is_empty() {
            out.push((true, "E742", u.file_path.clone(), format!("{q} is a unit but satisfies no requirement")));
        }
        if !l1.iter().any(|v| v.contains(q) || v.iter().any(|x| satisfied.contains(x))) {
            out.push((true, "E743", u.file_path.clone(), format!("{q} is a unit but no unit-level (L1) test verifies it or a requirement it satisfies")));
        }
        if !in_component(u) {
            out.push((true, "E744", u.file_path.clone(), format!("{q} is a unit but no `level: component` part contains it")));
        }
    }
    out
}
