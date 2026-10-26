//! Architecture rules (`REQ-TRS-ARCH-001`..`-005`, GH #369): `ArchitectureRule` elements evaluated by
//! `validate` over a dependency graph derived from the model.
//!
//! The nodes are the `PartDef` and `Part` elements. An edge `X → Y` means `X` depends on `Y`:
//!
//! * `connections`: a `connections:` entry `from: a.p, to: b.q` makes the definition of `b` depend on the
//!   definition of `a` (the consumer depends on the producer); a single-segment endpoint is a boundary
//!   port of the owner and stands for the owner;
//! * `typedBy`: a component depends on the definition of each usage in its `features:` (and of each child
//!   `Part`), and a `Part` on its `typedBy` definition;
//! * `allocations`: an allocated source depends on its target (software on the hardware it runs on).
//!
//! Five kinds: `forbid-dependency` (no component chosen by `from` depends, transitively, on one chosen by
//! `to`), `independence` (the transitive closures of the two sets are disjoint), `allow-only` (a `from` component
//! depends directly only on `from` or `to`), `layering` (no direct dependency on a layer above; `layer:` on
//! parts) and `max-depth` (no dependency chain longer than `maxDepth`). A rule may carry `waivers:`, each with a
//! rationale and an expiry (`REQ-TRS-ARCH-006`..`-008`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_yaml::Value as Yaml;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::tagged_values as tv;

/// `(is_error, code, file, message)`.
pub type Finding = (bool, &'static str, String, String);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Via {
    Connections,
    TypedBy,
    Allocations,
}

impl Via {
    fn parse(s: &str) -> Option<Via> {
        match s {
            "connections" => Some(Via::Connections),
            "typedBy" => Some(Via::TypedBy),
            "allocations" => Some(Via::Allocations),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct ComponentSelector {
    package: Option<String>,
    tags: Option<Vec<String>>,
    domains: Option<Vec<String>>,
    stereotype: Option<tv::Selector>,
    layers: Option<Vec<String>>,
}

fn strings(v: &Yaml) -> Option<Vec<String>> {
    match v {
        Yaml::String(s) => Some(vec![s.clone()]),
        Yaml::Sequence(q) => q.iter().map(|x| x.as_str().map(str::to_string)).collect(),
        _ => None,
    }
}

fn parse_selector(v: &Yaml, side: &str, elements: &[RawElement], resolver: &Resolver) -> Result<ComponentSelector, String> {
    let Yaml::Mapping(m) = v else { return Err(format!("`{side}` must be a mapping of package, tag, domain, stereotype")) };
    if m.is_empty() {
        return Err(format!("`{side}` is empty and would choose every component"));
    }
    let mut sel = ComponentSelector::default();
    for (k, val) in m {
        let Some(k) = k.as_str() else { return Err(format!("`{side}` has a key that is not text")) };
        match k {
            "package" => sel.package = Some(val.as_str().ok_or_else(|| format!("`{side}.package` must be a qualified name"))?.to_string()),
            "tag" => sel.tags = Some(strings(val).ok_or_else(|| format!("`{side}.tag` must be a tag or a list of tags"))?),
            "domain" => {
                let d = strings(val).ok_or_else(|| format!("`{side}.domain` must be a domain or a list of domains"))?;
                if let Some(bad) = d.iter().find(|x| !["system", "hardware", "software"].contains(&x.as_str())) {
                    return Err(format!("`{side}.domain` '{bad}' is not system, hardware or software"));
                }
                sel.domains = Some(d);
            }
            "stereotype" => {
                let s = tv::parse_selector(val).map_err(|e| format!("`{side}.stereotype`: {e}"))?;
                if tv::find_metadata_def(elements, resolver, &s.def).is_none() {
                    return Err(format!("`{side}.stereotype` names '{}', which is not a MetadataDef", s.def));
                }
                sel.stereotype = Some(s);
            }
            "layer" => sel.layers = Some(strings(val).ok_or_else(|| format!("`{side}.layer` must be a layer or a list of layers"))?),
            other => return Err(format!("`{side}` has an unknown key '{other}' (known: package, tag, domain, stereotype, layer)")),
        }
    }
    Ok(sel)
}

impl ComponentSelector {
    fn matches(&self, c: &RawElement, elements: &[RawElement], resolver: &Resolver) -> bool {
        let fm = &c.frontmatter;
        self.package.as_ref().is_none_or(|p| c.qualified_name == *p || c.qualified_name.starts_with(&format!("{p}::")))
            && self.tags.as_ref().is_none_or(|ts| ts.iter().any(|t| fm.tags.iter().flatten().any(|x| x == t)))
            && self.layers.as_ref().is_none_or(|ls| fm.layer.as_ref().is_some_and(|l| ls.contains(l)))
            && self.domains.as_ref().is_none_or(|ds| fm.domain.as_ref().is_some_and(|d| ds.contains(d)))
            && self.stereotype.as_ref().is_none_or(|s| tv::selector_matches(elements, resolver, c, s))
    }
}

/// The dependency graph: `deps[x]` are the components `x` depends on, with the edge source.
pub(crate) struct Graph {
    pub(crate) names: Vec<String>,
    pub(crate) deps: Vec<BTreeMap<usize, Via>>,
}

fn is_component(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef | ElementType::Part)) && !e.file_path.starts_with(tv::BUILTIN_PREFIX)
}

pub(crate) fn build_graph(elements: &[RawElement], resolver: &Resolver, via: &BTreeSet<Via>) -> (Graph, BTreeMap<String, usize>) {
    let comps: Vec<&RawElement> = elements.iter().filter(|e| is_component(e)).collect();
    let index: BTreeMap<String, usize> = comps.iter().enumerate().map(|(i, c)| (c.qualified_name.clone(), i)).collect();
    let mut g = Graph { names: comps.iter().map(|c| c.qualified_name.clone()).collect(), deps: vec![BTreeMap::new(); comps.len()] };
    let comp_of = |r: &str| -> Option<usize> { resolver.resolve_ref(elements, r).and_then(|e| index.get(&e.qualified_name).copied()) };
    let add = |x: usize, y: usize, v: Via, g: &mut Graph| {
        if x != y && via.contains(&v) {
            g.deps[x].entry(y).or_insert(v);
        }
    };

    for (i, c) in comps.iter().enumerate() {
        // usages: inline `features:` entries and child `Part` elements, name → definition
        let mut usages: BTreeMap<String, usize> = BTreeMap::new();
        for f in c.frontmatter.features.iter().flatten() {
            let (Some(name), Some(t)) = (f.get("name").and_then(Yaml::as_str), f.get("typedBy").or_else(|| f.get("type")).and_then(Yaml::as_str)) else { continue };
            if let Some(d) = comp_of(t) {
                usages.insert(name.to_string(), d);
            }
        }
        let prefix = format!("{}::", c.qualified_name);
        for u in elements.iter().filter(|u| is_component(u) && u.qualified_name.starts_with(&prefix) && u.frontmatter.element_type == Some(ElementType::Part)) {
            let name = &u.qualified_name[prefix.len()..];
            if let Some(d) = u.frontmatter.typed_by.as_ref().and_then(|t| strings(t)).and_then(|t| t.first().and_then(|t| comp_of(t))) {
                usages.insert(name.to_string(), d);
            }
        }
        // typedBy: a composite depends on what it contains; a Part on its definition
        for d in usages.values() {
            add(i, *d, Via::TypedBy, &mut g);
        }
        if c.frontmatter.element_type == Some(ElementType::Part) {
            if let Some(d) = c.frontmatter.typed_by.as_ref().and_then(|t| strings(t)).and_then(|t| t.first().and_then(|t| comp_of(t))) {
                add(i, d, Via::TypedBy, &mut g);
            }
        }
        // connections: the consumer (`to`) depends on the producer (`from`)
        let end = |chain: &str| -> Option<usize> {
            let first = chain.trim().split('.').next()?;
            if chain.contains('.') { usages.get(first).copied() } else { usages.get(first).copied().or(Some(i)) }
        };
        for entry in c.frontmatter.connections.iter().flatten().chain(c.frontmatter.flow_connections.iter().flatten()) {
            let Some(p) = crate::connections::parse_entry(entry) else { continue };
            if p.endpoints.len() < 2 {
                continue;
            }
            if let (Some(producer), Some(consumer)) = (end(&p.endpoints[0].chain), end(&p.endpoints[1].chain)) {
                add(consumer, producer, Via::Connections, &mut g);
            }
        }
        // allocations: the allocated source depends on its target
        for t in c.frontmatter.allocated_to.iter().flatten() {
            if let Some(y) = comp_of(t) {
                add(i, y, Via::Allocations, &mut g);
            }
        }
        for s in c.frontmatter.allocated_from.iter().flatten() {
            if let Some(x) = comp_of(s) {
                add(x, i, Via::Allocations, &mut g);
            }
        }
    }
    // `Allocation` elements (and Allocation entries in a `features:` list)
    for e in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::Allocation)) {
        let mut pairs: Vec<(String, String)> = Vec::new();
        for f in e.frontmatter.features.iter().flatten() {
            if let (Some(a), Some(b)) = (f.get("allocatedFrom").and_then(Yaml::as_str), f.get("allocatedTo").and_then(Yaml::as_str)) {
                pairs.push((a.to_string(), b.to_string()));
            }
        }
        for a in e.frontmatter.allocated_from.iter().flatten() {
            for b in e.frontmatter.allocated_to.iter().flatten() {
                pairs.push((a.clone(), b.clone()));
            }
        }
        for (a, b) in pairs {
            if let (Some(x), Some(y)) = (comp_of(&a), comp_of(&b)) {
                add(x, y, Via::Allocations, &mut g);
            }
        }
    }
    (g, index)
}

/// Breadth-first from `sources`: the reached nodes with the node each was reached from. A node in `stop` is
/// reached but not searched through.
fn reach_stopping(g: &Graph, sources: &[usize], stop: &BTreeSet<usize>) -> BTreeMap<usize, Option<usize>> {
    let mut seen: BTreeMap<usize, Option<usize>> = BTreeMap::new();
    let mut q = VecDeque::new();
    for &s in sources {
        if seen.insert(s, None).is_none() {
            q.push_back(s);
        }
    }
    while let Some(x) = q.pop_front() {
        // a source is always searched; any other node in `stop` ends the path
        if stop.contains(&x) && !sources.contains(&x) {
            continue;
        }
        for &y in g.deps[x].keys() {
            if !seen.contains_key(&y) {
                seen.insert(y, Some(x));
                q.push_back(y);
            }
        }
    }
    seen
}

fn reach(g: &Graph, sources: &[usize]) -> BTreeMap<usize, Option<usize>> {
    reach_stopping(g, sources, &BTreeSet::new())
}

fn path(g: &Graph, parents: &BTreeMap<usize, Option<usize>>, to: usize) -> String {
    let mut p = vec![to];
    while let Some(Some(prev)) = parents.get(p.last().unwrap()) {
        p.push(*prev);
    }
    p.reverse();
    p.iter().map(|i| g.names[*i].as_str()).collect::<Vec<_>>().join(" → ")
}

/// Evaluate every `ArchitectureRule` of the model.
pub fn check(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for rule in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::ArchitectureRule)) {
        let fm = &rule.frontmatter;
        let file = rule.file_path.clone();
        let label = fm.id.clone().unwrap_or_else(|| rule.qualified_name.clone());
        let mut bad: Vec<String> = Vec::new();
        match fm.id.as_deref() {
            Some(id) if crate::resolver::is_ar_id(id) => {}
            Some(id) => bad.push(format!("id '{id}' is not an AR-* id (^AR(-[A-Z0-9]{{2,12}})*-[0-9]{{3,8}}$)")),
            None => bad.push("`id` is required on an ArchitectureRule".into()),
        }
        if fm.name.is_none() {
            bad.push("`name` is required on an ArchitectureRule".into());
        }
        let kind = fm.kind.as_deref();
        if !matches!(kind, Some("forbid-dependency" | "independence" | "allow-only" | "layering" | "max-depth")) {
            bad.push(format!(
                "`kind` {} must be forbid-dependency, independence, allow-only, layering or max-depth",
                kind.map_or("is missing and".to_string(), |k| format!("'{k}'"))
            ));
        }
        let need_from = !matches!(kind, Some("layering" | "max-depth"));
        let need_to = matches!(kind, Some("forbid-dependency" | "independence" | "allow-only"));
        let mut layer_list: Vec<String> = Vec::new();
        if kind == Some("layering") {
            match fm.layers.as_ref().filter(|v| v.is_sequence()).and_then(strings) {
                Some(l) if !l.is_empty() && l.iter().collect::<BTreeSet<_>>().len() == l.len() => layer_list = l,
                _ => bad.push("`layers` must be a non-empty list of distinct layer names, top first".into()),
            }
        }
        let mut max_depth = 0usize;
        if kind == Some("max-depth") {
            match fm.max_depth.as_ref().and_then(Yaml::as_u64) {
                Some(n) => max_depth = n as usize,
                None => bad.push("`maxDepth` must be a non-negative integer".into()),
            }
        }
        let waivers = parse_waivers(fm.waivers.as_ref(), &mut bad);
        let mut via: BTreeSet<Via> = [Via::Connections, Via::TypedBy, Via::Allocations].into();
        if let Some(v) = &fm.via {
            let parsed: Vec<Option<Via>> = v.iter().map(|s| Via::parse(s)).collect();
            if v.is_empty() || parsed.iter().any(Option::is_none) {
                bad.push("`via` must be a non-empty list of connections, typedBy, allocations".into());
            } else {
                via = parsed.into_iter().flatten().collect();
            }
        }
        let is_warning = match fm.severity.as_deref() {
            None | Some("error") => false,
            Some("warning") => true,
            Some(other) => {
                bad.push(format!("`severity` '{other}' must be error or warning"));
                false
            }
        };
        let mut sels: Vec<Option<ComponentSelector>> = Vec::new();
        for (side, v, need) in [("from", &fm.from, need_from), ("to", &fm.to, need_to)] {
            match v {
                None => {
                    if need {
                        bad.push(format!("`{side}` is required"));
                    }
                    sels.push(None);
                }
                Some(v) => match parse_selector(v, side, elements, resolver) {
                    Ok(s) => sels.push(Some(s)),
                    Err(e) => {
                        bad.push(e);
                        sels.push(None);
                    }
                },
            }
        }
        for b in &bad {
            out.push((true, "E662", file.clone(), format!("ArchitectureRule '{label}': {b}")));
        }
        if !bad.is_empty() || fm.status.as_deref() == Some("draft") {
            continue;
        }
        let (from_sel, to_sel) = (sels[0].take(), sels[1].take());
        let (g, index) = build_graph(elements, resolver, &via);
        let choose = |s: &ComponentSelector| -> Vec<usize> {
            elements.iter().filter(|e| is_component(e) && s.matches(e, elements, resolver)).filter_map(|e| index.get(&e.qualified_name).copied()).collect()
        };
        let all = || -> Vec<usize> { elements.iter().filter(|e| is_component(e)).filter_map(|e| index.get(&e.qualified_name).copied()).collect() };
        let a = from_sel.as_ref().map(&choose).unwrap_or_else(all);
        let b = to_sel.as_ref().map(&choose).unwrap_or_default();
        let mut vacuous = false;
        for (side, set, given) in [("from", &a, from_sel.is_some()), ("to", &b, to_sel.is_some())] {
            if given && set.is_empty() {
                vacuous = true;
                out.push((false, "W662", file.clone(), format!("ArchitectureRule '{label}': `{side}` chooses no component, so the rule checks nothing")));
            }
        }
        if vacuous {
            continue;
        }
        let rname = fm.name.clone().unwrap_or_default();
        // (subject for waivers, is_error, code, message)
        let mut cands: Vec<(String, bool, &'static str, String)> = Vec::new();
        match kind {
            Some("forbid-dependency") => {
                let (code, err) = if is_warning { ("W660", false) } else { ("E660", true) };
                // A path ends at the first forbidden component and does not run through another `from` component
                // (that one reports for itself), so one wiring mistake is one finding.
                let stop: BTreeSet<usize> = a.iter().chain(b.iter()).copied().collect();
                for &x in &a {
                    let seen = reach_stopping(&g, &[x], &stop);
                    for &y in &b {
                        if y != x && seen.contains_key(&y) {
                            cands.push((g.names[x].clone(), err, code, format!("ArchitectureRule '{label}' ({rname}): forbidden dependency {}", path(&g, &seen, y))));
                        }
                    }
                }
            }
            Some("allow-only") => {
                let (code, err) = if is_warning { ("W660", false) } else { ("E660", true) };
                for &x in &a {
                    for &y in g.deps[x].keys() {
                        if y != x && !a.contains(&y) && !b.contains(&y) {
                            cands.push((
                                g.names[x].clone(),
                                err,
                                code,
                                format!("ArchitectureRule '{label}' ({rname}): {} depends on {}, which is neither in `from` nor in `to`", g.names[x], g.names[y]),
                            ));
                        }
                    }
                }
            }
            Some("layering") => {
                let (code, err) = if is_warning { ("W660", false) } else { ("E660", true) };
                let layer_of: BTreeMap<&str, usize> = elements
                    .iter()
                    .filter(|e| is_component(e))
                    .filter_map(|e| Some((e.qualified_name.as_str(), layer_list.iter().position(|l| Some(l) == e.frontmatter.layer.as_ref())?)))
                    .collect();
                for &x in &a {
                    let Some(&lx) = layer_of.get(g.names[x].as_str()) else { continue };
                    for &y in g.deps[x].keys() {
                        if let Some(&ly) = layer_of.get(g.names[y].as_str()) {
                            if ly < lx {
                                cands.push((
                                    g.names[x].clone(),
                                    err,
                                    code,
                                    format!(
                                        "ArchitectureRule '{label}' ({rname}): {} (layer {}) depends on {} (layer {}), a layer above it",
                                        g.names[x], layer_list[lx], g.names[y], layer_list[ly]
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            Some("max-depth") => {
                let (code, err) = if is_warning { ("W663", false) } else { ("E663", true) };
                let mut depth: BTreeMap<usize, (usize, Option<usize>)> = BTreeMap::new();
                let mut on_stack: BTreeSet<usize> = BTreeSet::new();
                for &x in &a {
                    longest(&g, x, &mut depth, &mut on_stack);
                }
                let mut starts: Vec<(usize, usize)> = a.iter().map(|&x| (x, depth[&x].0)).filter(|(_, d)| *d > max_depth).collect();
                starts.sort_by(|p, q| q.1.cmp(&p.1).then_with(|| g.names[p.0].cmp(&g.names[q.0])));
                let mut covered: BTreeSet<usize> = BTreeSet::new();
                for (x, d) in starts {
                    if covered.contains(&x) {
                        continue;
                    }
                    let mut chain = vec![x];
                    let mut cur = x;
                    while let Some(n) = depth[&cur].1 {
                        if chain.contains(&n) {
                            break;
                        }
                        chain.push(n);
                        cur = n;
                    }
                    covered.extend(chain.iter().copied());
                    let text = chain.iter().map(|&i| g.names[i].as_str()).collect::<Vec<_>>().join(" → ");
                    cands.push((g.names[x].clone(), err, code, format!("ArchitectureRule '{label}' ({rname}): {} starts a dependency chain of depth {d}, above the limit {max_depth}: {text}", g.names[x])));
                }
            }
            _ => {
                let (code, err) = if is_warning { ("W661", false) } else { ("E661", true) };
                let (ra, rb) = (reach(&g, &a), reach(&g, &b));
                let shared: Vec<usize> = ra.keys().filter(|k| rb.contains_key(k)).copied().collect();
                // Report the components that cause the sharing, not what they contain or depend on: one that is
                // reachable from another shared component (which does not depend back on it) is a consequence.
                let below: BTreeMap<usize, BTreeMap<usize, Option<usize>>> = shared.iter().map(|&t| (t, reach(&g, &[t]))).collect();
                for &s in &shared {
                    let consequence = shared.iter().any(|&t| t != s && below[&t].contains_key(&s) && !below[&s].contains_key(&t));
                    if consequence {
                        continue;
                    }
                    cands.push((
                        g.names[s].clone(),
                        err,
                        code,
                        format!("ArchitectureRule '{label}' ({rname}): {} is shared — from: {}; to: {}", g.names[s], path(&g, &ra, s), path(&g, &rb, s)),
                    ));
                }
            }
        }
        // Waivers: a current one suppresses the findings whose subject it names; an expired one does not, and
        // either kind that matches nothing is reported.
        let today = crate::residual_risk::today();
        for w in &waivers {
            let expired = w.expires.as_str() <= today.as_str();
            let hits = cands.iter().filter(|c| c.0 == w.element).count();
            if expired {
                out.push((false, "W664", file.clone(), format!("ArchitectureRule '{label}': the waiver for '{}' expired on {}", w.element, w.expires)));
            } else if hits == 0 {
                out.push((false, "W664", file.clone(), format!("ArchitectureRule '{label}': the waiver for '{}' suppresses nothing", w.element)));
            } else {
                cands.retain(|c| c.0 != w.element);
            }
        }
        for (_, err, code, msg) in cands {
            out.push((err, code, file.clone(), msg));
        }
    }
    out
}

struct Waiver {
    element: String,
    expires: String,
}

/// The `waivers:` of a rule; a malformed entry is pushed to `bad` and skipped.
fn parse_waivers(v: Option<&Yaml>, bad: &mut Vec<String>) -> Vec<Waiver> {
    let Some(v) = v else { return Vec::new() };
    let Yaml::Sequence(items) = v else {
        bad.push("`waivers` must be a list of {element, rationale, expires}".into());
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, it) in items.iter().enumerate() {
        let get = |k: &str| it.get(k).and_then(Yaml::as_str).map(str::trim).filter(|s| !s.is_empty());
        let (Some(element), Some(_), Some(expires)) = (get("element"), get("rationale"), get("expires")) else {
            bad.push(format!("`waivers[{i}]` needs an `element`, a `rationale` and an `expires` date"));
            continue;
        };
        if !valid_date(expires) {
            bad.push(format!("`waivers[{i}].expires` '{expires}' is not a date YYYY-MM-DD"));
            continue;
        }
        out.push(Waiver { element: element.to_string(), expires: expires.to_string() });
    }
    out
}

fn valid_date(s: &str) -> bool {
    let p: Vec<&str> = s.split('-').collect();
    p.len() == 3
        && p[0].len() == 4
        && p[1].len() == 2
        && p[2].len() == 2
        && matches!((p[0].parse::<u32>(), p[1].parse::<u32>(), p[2].parse::<u32>()), (Ok(_), Ok(1..=12), Ok(1..=31)))
}

/// Longest chain of dependency edges starting at `x`, with the next component on it. Each component is visited
/// once per chain (a component on the current chain is skipped), and a result is memoised.
fn longest(g: &Graph, x: usize, memo: &mut BTreeMap<usize, (usize, Option<usize>)>, on_stack: &mut BTreeSet<usize>) -> usize {
    if let Some(&(d, _)) = memo.get(&x) {
        return d;
    }
    on_stack.insert(x);
    let mut best = (0usize, None);
    for &y in g.deps[x].keys() {
        if y == x || on_stack.contains(&y) {
            continue;
        }
        let d = longest(g, y, memo, on_stack) + 1;
        if d > best.0 {
            best = (d, Some(y));
        }
    }
    on_stack.remove(&x);
    memo.insert(x, best);
    best.0
}
