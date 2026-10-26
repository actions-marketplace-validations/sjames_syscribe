//! `meta` — the stereotype report (`REQ-TRS-META-052`, GH #379): which elements apply a stereotype,
//! grouped by one of its tagged values. The compliance worklist (`Regulatory` by `status`) is one call.
//! The MCP `meta` tool (`REQ-TRS-META-062`) returns the same data: [`report`] computes it, [`Report::json`]
//! and [`Report::print`] render it.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::tagged_values as tv;

/// What `meta` found.
pub enum Report {
    /// No `--def`: the applications of every stereotype, most first.
    Definitions(Vec<(String, usize)>),
    /// `--def` alone: the elements that apply it (or a specialisation).
    Def { def: String, count: usize, elements: Vec<String> },
    /// `--def` with `--group-by`: the same, grouped by a tagged value, largest group first.
    Grouped { def: String, group_by: String, count: usize, groups: Vec<(Option<String>, Vec<String>)> },
}

fn value_text(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

/// Compute the report. `Err` is a usage error: an unknown `def`.
pub fn report(elements: &[RawElement], def_name: Option<&str>, group_by: Option<&str>) -> Result<Report, String> {
    let resolver = Resolver::new(elements);
    let carriers: Vec<&RawElement> = elements.iter().filter(|e| !e.file_path.starts_with(tv::BUILTIN_PREFIX)).collect();

    let Some(def_name) = def_name else {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for e in &carriers {
            for a in tv::effective_applications(elements, &resolver, e) {
                *counts.entry(a.def).or_default() += 1;
            }
        }
        let mut rows: Vec<(String, usize)> = counts.into_iter().collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        return Ok(Report::Definitions(rows));
    };

    let Some(def) = tv::find_metadata_def(elements, &resolver, def_name) else {
        return Err(format!("--def '{def_name}' is not a MetadataDef of the model"));
    };
    let want = def.qualified_name.clone();
    // (group value, carrier) for every application of the definition or a specialisation of it.
    let mut groups: BTreeMap<Option<String>, Vec<String>> = BTreeMap::new();
    let mut total = 0usize;
    for e in &carriers {
        for a in tv::effective_applications(elements, &resolver, e).into_iter().filter(|a| a.lineage.contains(&want)) {
            total += 1;
            let key = group_by.map(|k| a.values.iter().find(|(n, _)| n == k).map(|(_, v)| value_text(v)));
            groups.entry(key.flatten()).or_default().push(e.qualified_name.clone());
        }
    }
    for v in groups.values_mut() {
        v.sort();
        v.dedup();
    }
    let mut ordered: Vec<(Option<String>, Vec<String>)> = groups.into_iter().collect();
    ordered.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
    Ok(match group_by {
        None => Report::Def { def: want, count: total, elements: ordered.into_iter().flat_map(|(_, v)| v).collect() },
        Some(k) => Report::Grouped { def: want, group_by: k.to_string(), count: total, groups: ordered },
    })
}

impl Report {
    pub fn json(&self) -> Value {
        match self {
            Report::Definitions(rows) => json!({"definitions": rows.iter().map(|(d, n)| json!({"def": d, "count": n})).collect::<Vec<_>>()}),
            Report::Def { def, count, elements } => json!({"def": def, "count": count, "elements": elements}),
            Report::Grouped { def, group_by, count, groups } => json!({
                "def": def,
                "groupBy": group_by,
                "count": count,
                "groups": groups.iter().map(|(v, e)| json!({"value": v, "count": e.len(), "elements": e})).collect::<Vec<_>>(),
            }),
        }
    }

    pub fn print(&self) {
        match self {
            Report::Definitions(rows) if rows.is_empty() => println!("No element applies a stereotype."),
            Report::Definitions(rows) => {
                println!("| Stereotype | Applications |\n|---|---|");
                for (d, n) in rows {
                    println!("| {d} | {n} |");
                }
            }
            Report::Def { def, count, elements } => {
                println!("# {def}\n\n{count} application(s)\n");
                for q in elements {
                    println!("- {q}");
                }
            }
            Report::Grouped { def, group_by, count, groups } => {
                println!("# {def} by {group_by}\n\n{count} application(s)\n");
                for (v, e) in groups {
                    println!("## {} ({})\n", v.as_deref().unwrap_or("(unset)"), e.len());
                    for q in e {
                        println!("- {q}");
                    }
                    println!();
                }
            }
        }
    }
}

fn flag<'a>(rest: &'a [String], name: &str) -> Option<&'a str> {
    rest.windows(2).find(|w| w[0] == name).map(|w| w[1].as_str())
}

pub fn cmd_meta(elements: &[RawElement], rest: &[String]) -> i32 {
    match report(elements, flag(rest, "--def"), flag(rest, "--group-by")) {
        Ok(r) => {
            if rest.iter().any(|a| a == "--json") {
                println!("{}", serde_json::to_string_pretty(&r.json()).unwrap());
            } else {
                r.print();
            }
            0
        }
        Err(m) => {
            eprintln!("Error: {m}");
            1
        }
    }
}
