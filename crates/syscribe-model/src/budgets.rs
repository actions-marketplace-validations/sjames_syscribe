//! Resource budgets (`REQ-TRS-BUD-001`..`-003`, GH #368): the static memory declared on software parts, the capacity
//! and margin on the hardware part they are allocated to, and the roll-up check in `validate`.
//!
//! Opt-in: a part that declares none of the fields is not looked at, and a hardware part with no capacity for a
//! resource is not checked for it.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_yaml::Value as Yaml;

use crate::arch_rules::{build_graph, Via};
use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// `(is_error, code, file, message)`.
pub type Finding = (bool, &'static str, String, String);

/// A memory size in bytes: a number, or text with a unit (`B`, `kB`/`MB`/`GB` by 1000, `KiB`/`MiB`/`GiB` by 1024).
pub fn parse_size(v: &Yaml) -> Result<f64, String> {
    let (num, unit) = match v {
        Yaml::Number(n) => return n.as_f64().filter(|x| *x >= 0.0).ok_or_else(|| "must not be negative".to_string()),
        Yaml::String(s) => {
            let t = s.trim();
            let split = t.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(t.len());
            (t[..split].trim().to_string(), t[split..].trim().to_string())
        }
        _ => return Err("must be a size such as `48 KiB`".into()),
    };
    let x: f64 = num.parse().map_err(|_| format!("'{}' is not a size such as `48 KiB`", match v { Yaml::String(s) => s.as_str(), _ => "" }))?;
    if x < 0.0 {
        return Err("must not be negative".into());
    }
    let mult = match unit.as_str() {
        "" | "B" => 1.0,
        "kB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "KB" | "kb" | "Kb" | "KIB" | "kib" => return Err(format!("unit '{unit}' is ambiguous — use kB (1000 bytes) or KiB (1024 bytes)")),
        other => return Err(format!("unit '{other}' is not B, kB, MB, GB, KiB, MiB or GiB")),
    };
    Ok(x * mult)
}

/// A margin: a percentage from 0 to 100, as a number or `"80%"`.
pub fn parse_margin(v: &Yaml) -> Result<f64, String> {
    let x = match v {
        Yaml::Number(n) => n.as_f64(),
        Yaml::String(s) => s.trim().trim_end_matches('%').trim().parse().ok(),
        _ => None,
    };
    x.filter(|p| (0.0..=100.0).contains(p)).ok_or_else(|| "must be a percentage from 0 to 100".to_string())
}

/// `[budgets] margin` of `.syscribe.toml` (a percentage), else 100.
pub fn project_margin(model_root: Option<&std::path::Path>) -> f64 {
    model_root
        .and_then(|r| std::fs::read_to_string(r.join(".syscribe.toml")).ok())
        .and_then(|t| toml::from_str::<toml::Value>(&t).ok())
        .and_then(|v| v.get("budgets")?.get("margin").cloned())
        .and_then(|m| m.as_float().or_else(|| m.as_integer().map(|i| i as f64)))
        .filter(|p| (0.0..=100.0).contains(p))
        .unwrap_or(100.0)
}

/// One resource of one hardware part.
#[derive(Debug, Clone)]
pub struct Row {
    pub element: String,
    pub file: String,
    pub resource: &'static str,
    pub used: f64,
    pub capacity: f64,
    pub margin: f64,
    /// Largest first.
    pub contributors: Vec<(String, f64)>,
}

impl Row {
    pub fn limit(&self) -> f64 {
        self.capacity * self.margin / 100.0
    }
    pub fn utilisation(&self) -> f64 {
        if self.capacity > 0.0 { self.used / self.capacity } else { 0.0 }
    }
}

/// `81000` → `81 kB`.
pub fn human(bytes: f64) -> String {
    for (unit, div) in [("GB", 1e9), ("MB", 1e6), ("kB", 1e3)] {
        if bytes >= div {
            let v = bytes / div;
            return format!("{} {unit}", crate::expr::format_number((v * 1000.0).round() / 1000.0));
        }
    }
    format!("{} B", crate::expr::format_number(bytes))
}

fn is_part(e: &RawElement) -> bool {
    matches!(e.frontmatter.element_type, Some(ElementType::PartDef | ElementType::Part))
}

/// Every row, and the `E691` findings for malformed values.
pub fn compute(elements: &[RawElement], resolver: &Resolver, default_margin: f64) -> (Vec<Row>, Vec<Finding>) {
    let mut findings: Vec<Finding> = Vec::new();
    let size = |e: &RawElement, name: &str, v: &Option<Yaml>, findings: &mut Vec<Finding>| -> Option<f64> {
        let v = v.as_ref()?;
        match parse_size(v) {
            Ok(b) => Some(b),
            Err(why) => {
                findings.push((true, "E691", e.file_path.clone(), format!("'{}': {name} {why}", e.qualified_name)));
                None
            }
        }
    };

    // own use of each part, and capacities
    let mut own: BTreeMap<String, [f64; 3]> = BTreeMap::new(); // ram(+stack), flash, nvm
    let mut caps: Vec<(&RawElement, [Option<f64>; 3], f64)> = Vec::new();
    for e in elements.iter().filter(|e| is_part(e)) {
        let fm = &e.frontmatter;
        let ram = size(e, "ram", &fm.ram, &mut findings);
        let stack = size(e, "stack", &fm.stack, &mut findings);
        let flash = size(e, "flash", &fm.flash, &mut findings);
        let nvm = size(e, "nvm", &fm.nvm, &mut findings);
        if ram.is_some() || stack.is_some() || flash.is_some() || nvm.is_some() {
            own.insert(e.qualified_name.clone(), [ram.unwrap_or(0.0) + stack.unwrap_or(0.0), flash.unwrap_or(0.0), nvm.unwrap_or(0.0)]);
        }
        let c = [
            size(e, "ramCapacity", &fm.ram_capacity, &mut findings),
            size(e, "flashCapacity", &fm.flash_capacity, &mut findings),
            size(e, "nvmCapacity", &fm.nvm_capacity, &mut findings),
        ];
        let margin = match fm.margin.as_ref().map(parse_margin) {
            Some(Ok(m)) => m,
            Some(Err(why)) => {
                findings.push((true, "E691", e.file_path.clone(), format!("'{}': margin {why}", e.qualified_name)));
                default_margin
            }
            None => default_margin,
        };
        if c.iter().any(Option::is_some) {
            caps.push((e, c, margin));
        }
    }
    if caps.is_empty() {
        return (Vec::new(), findings);
    }

    // who is allocated to what, and what contains what
    let alloc: BTreeSet<Via> = [Via::Allocations].into();
    let contain: BTreeSet<Via> = [Via::TypedBy].into();
    let (ga, index) = build_graph(elements, resolver, &alloc);
    let (gc, _) = build_graph(elements, resolver, &contain);

    let mut rows = Vec::new();
    for (hw, c, margin) in caps {
        let Some(&h) = index.get(&hw.qualified_name) else { continue };
        // components allocated to this part, and what they contain, each once
        let mut seen: BTreeSet<usize> = ga.deps.iter().enumerate().filter(|(_, d)| d.contains_key(&h)).map(|(i, _)| i).collect();
        let mut q: VecDeque<usize> = seen.iter().copied().collect();
        while let Some(x) = q.pop_front() {
            for &y in gc.deps[x].keys() {
                if seen.insert(y) {
                    q.push_back(y);
                }
            }
        }
        for (k, resource) in ["ram", "flash", "nvm"].into_iter().enumerate() {
            let Some(capacity) = c[k] else { continue };
            let mut contributors: Vec<(String, f64)> = seen
                .iter()
                .filter_map(|&i| own.get(&ga.names[i]).map(|u| (ga.names[i].clone(), u[k])))
                .filter(|(_, b)| *b > 0.0)
                .collect();
            contributors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(&b.0)));
            rows.push(Row { element: hw.qualified_name.clone(), file: hw.file_path.clone(), resource, used: contributors.iter().map(|c| c.1).sum::<f64>() + 0.0, capacity, margin, contributors });
        }
    }
    (rows, findings)
}

pub fn check(elements: &[RawElement], resolver: &Resolver, default_margin: f64) -> Vec<Finding> {
    let (rows, mut findings) = compute(elements, resolver, default_margin);
    for r in rows.iter().filter(|r| r.used > r.limit() + 1e-9) {
        let who = r.contributors.iter().map(|(n, b)| format!("{n} {}", human(*b))).collect::<Vec<_>>().join(", ");
        findings.push((
            true,
            "E690",
            r.file.clone(),
            format!(
                "{}: {} use {} exceeds the limit {} ({}% of the {} capacity); contributors: {who}",
                r.element,
                r.resource,
                human(r.used),
                human(r.limit()),
                crate::expr::format_number(r.margin),
                human(r.capacity)
            ),
        ));
    }
    findings
}
