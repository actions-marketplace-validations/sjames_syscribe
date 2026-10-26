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
        let seen = reach(&ga, &gc, h);
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

/// The components allocated to `h`, and everything they contain, each once.
fn reach(ga: &crate::arch_rules::Graph, gc: &crate::arch_rules::Graph, h: usize) -> BTreeSet<usize> {
    let mut seen: BTreeSet<usize> = ga.deps.iter().enumerate().filter(|(_, d)| d.contains_key(&h)).map(|(i, _)| i).collect();
    let mut q: VecDeque<usize> = seen.iter().copied().collect();
    while let Some(x) = q.pop_front() {
        for &y in gc.deps[x].keys() {
            if seen.insert(y) {
                q.push_back(y);
            }
        }
    }
    seen
}

/// A positive duration in seconds: text with a time unit.
fn parse_duration(v: &Yaml) -> Result<f64, String> {
    let Yaml::String(t) = v else { return Err("is not a duration with a unit, such as \"10 ms\"".into()) };
    match crate::system::duration(t) {
        Some(x) if x > 0.0 => Ok(x),
        Some(_) => Err("must be positive".into()),
        None => Err(format!("'{t}' is not a duration with a unit, such as \"10 ms\"")),
    }
}

/// One schedulable task: a software part with a `period` and a `wcet` duration.
pub struct Task {
    pub element: String,
    pub wcet: f64,
    pub period: f64,
}

impl Task {
    pub fn utilisation(&self) -> f64 {
        self.wcet / self.period
    }
}

/// CPU utilisation of what is allocated to a hardware part that opts in with `cpuMargin` or `scheduling`.
pub struct CpuRow {
    pub element: String,
    pub file: String,
    pub scheduling: &'static str,
    pub margin: f64,
    pub tasks: Vec<Task>,
}

impl CpuRow {
    pub fn utilisation(&self) -> f64 {
        self.tasks.iter().map(Task::utilisation).sum::<f64>() + 0.0
    }
    /// The schedulability bound as a fraction: 1 for EDF, Liu–Layland `n (2^(1/n) − 1)` for rate-monotonic.
    pub fn bound(&self) -> f64 {
        let n = self.tasks.len().max(1) as f64;
        if self.scheduling == "rate-monotonic" { n * (2f64.powf(1.0 / n) - 1.0) } else { 1.0 }
    }
    pub fn limit(&self) -> f64 {
        self.bound() * self.margin / 100.0
    }
}

/// The timing rows and findings: `E691` (malformed timing/cpu fields), `E693` (a part's `wcet` above its `deadline`).
pub fn compute_cpu(elements: &[RawElement], resolver: &Resolver, default_margin: f64) -> (Vec<CpuRow>, Vec<Finding>) {
    let _ = default_margin;
    let mut findings: Vec<Finding> = Vec::new();
    let mut tasks: BTreeMap<String, Task> = BTreeMap::new();
    let mut hws: Vec<(&RawElement, &'static str, f64)> = Vec::new();
    for e in elements.iter().filter(|e| is_part(e)) {
        let fm = &e.frontmatter;
        let mut bad = |what: &str, why: String| findings.push((true, "E691", e.file_path.clone(), format!("'{}': {what} {why}", e.qualified_name)));
        let period = fm.period.as_ref().and_then(|v| parse_duration(v).map_err(|w| bad("period", w)).ok());
        let deadline = fm.deadline.as_ref().and_then(|v| parse_duration(v).map_err(|w| bad("deadline", w)).ok());
        if let Some(p) = &fm.priority {
            if p.as_i64().is_none() {
                bad("priority", "is not an integer".into());
            }
        }
        // `wcet` on a part is read when it is a duration; an opaque claim is left alone
        let wcet = fm.wcet.as_deref().and_then(crate::system::duration);
        if let (Some(w), Some(d)) = (wcet, deadline) {
            if w > d + 1e-12 {
                findings.push((true, "E693", e.file_path.clone(), format!("{}: wcet {} exceeds its deadline {}", e.qualified_name, human_time(w), human_time(d))));
            }
        }
        if let (Some(w), Some(p)) = (wcet, period) {
            tasks.insert(e.qualified_name.clone(), Task { element: e.qualified_name.clone(), wcet: w, period: p });
        }
        let margin = match fm.cpu_margin.as_ref().map(parse_margin) {
            Some(Ok(m)) => Some(m),
            Some(Err(why)) => {
                findings.push((true, "E691", e.file_path.clone(), format!("'{}': cpuMargin {why}", e.qualified_name)));
                Some(100.0)
            }
            None => None,
        };
        let sched = match fm.scheduling.as_deref() {
            None => None,
            Some("edf") => Some("edf"),
            Some("rate-monotonic") => Some("rate-monotonic"),
            Some(other) => {
                findings.push((true, "E691", e.file_path.clone(), format!("'{}': scheduling '{other}' is not edf or rate-monotonic", e.qualified_name)));
                Some("edf")
            }
        };
        if margin.is_some() || sched.is_some() {
            hws.push((e, sched.unwrap_or("edf"), margin.unwrap_or(100.0)));
        }
    }
    if hws.is_empty() {
        return (Vec::new(), findings);
    }
    let alloc: BTreeSet<Via> = [Via::Allocations].into();
    let contain: BTreeSet<Via> = [Via::TypedBy].into();
    let (ga, index) = build_graph(elements, resolver, &alloc);
    let (gc, _) = build_graph(elements, resolver, &contain);
    let mut rows = Vec::new();
    for (hw, scheduling, margin) in hws {
        let Some(&h) = index.get(&hw.qualified_name) else { continue };
        let mut ts: Vec<Task> = reach(&ga, &gc, h)
            .iter()
            .filter_map(|&i| tasks.get(&ga.names[i]))
            .map(|t| Task { element: t.element.clone(), wcet: t.wcet, period: t.period })
            .collect();
        ts.sort_by(|a, b| b.utilisation().partial_cmp(&a.utilisation()).unwrap().then_with(|| a.element.cmp(&b.element)));
        rows.push(CpuRow { element: hw.qualified_name.clone(), file: hw.file_path.clone(), scheduling, margin, tasks: ts });
    }
    (rows, findings)
}

fn human_time(s: f64) -> String {
    for (unit, scale) in [("s", 1.0), ("ms", 1e-3), ("us", 1e-6)] {
        if s >= scale {
            return format!("{} {unit}", crate::expr::format_number((s / scale * 1e6).round() / 1e6));
        }
    }
    format!("{} s", crate::expr::format_number(s))
}

pub fn check(elements: &[RawElement], resolver: &Resolver, default_margin: f64) -> Vec<Finding> {
    let (rows, mut findings) = compute(elements, resolver, default_margin);
    let (cpu, more) = compute_cpu(elements, resolver, default_margin);
    findings.extend(more);
    for r in cpu.iter().filter(|r| r.utilisation() > r.limit() + 1e-9) {
        let who = r.tasks.iter().map(|t| format!("{} {:.1}%", t.element, t.utilisation() * 100.0)).collect::<Vec<_>>().join(", ");
        findings.push((
            true,
            "E692",
            r.file.clone(),
            format!(
                "{}: CPU utilisation {:.1}% exceeds the limit {:.1}% ({} bound {:.1}% x {}% margin); tasks: {who}",
                r.element,
                r.utilisation() * 100.0,
                r.limit() * 100.0,
                r.scheduling,
                r.bound() * 100.0,
                crate::expr::format_number(r.margin)
            ),
        ));
    }
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
