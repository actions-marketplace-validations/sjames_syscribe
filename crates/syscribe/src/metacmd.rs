//! `meta` — the stereotype report (`REQ-TRS-META-052`, GH #379): which elements apply a stereotype,
//! grouped by one of its tagged values. The compliance worklist (`Regulatory` by `status`) is one call.

use std::collections::BTreeMap;

use syscribe_model::element::RawElement;
use syscribe_model::resolver::Resolver;
use syscribe_model::tagged_values as tv;

fn flag<'a>(rest: &'a [String], name: &str) -> Option<&'a str> {
    rest.windows(2).find(|w| w[0] == name).map(|w| w[1].as_str())
}

fn value_text(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

pub fn cmd_meta(elements: &[RawElement], rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    let resolver = Resolver::new(elements);
    let carriers: Vec<&RawElement> = elements.iter().filter(|e| !e.file_path.starts_with(tv::BUILTIN_PREFIX)).collect();

    let Some(def_name) = flag(rest, "--def") else {
        // Applications per definition.
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for e in &carriers {
            for a in tv::effective_applications(elements, &resolver, e) {
                *counts.entry(a.def).or_default() += 1;
            }
        }
        let mut rows: Vec<(String, usize)> = counts.into_iter().collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        if json {
            let defs: Vec<_> = rows.iter().map(|(d, n)| serde_json::json!({"def": d, "count": n})).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({"definitions": defs})).unwrap());
        } else if rows.is_empty() {
            println!("No element applies a stereotype.");
        } else {
            println!("| Stereotype | Applications |\n|---|---|");
            for (d, n) in &rows {
                println!("| {d} | {n} |");
            }
        }
        return 0;
    };

    let Some(def) = tv::find_metadata_def(elements, &resolver, def_name) else {
        eprintln!("Error: --def '{def_name}' is not a MetadataDef of the model");
        return 1;
    };
    let want = def.qualified_name.clone();
    let group_by = flag(rest, "--group-by");
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

    if json {
        let out = match group_by {
            None => serde_json::json!({"def": want, "count": total, "elements": ordered.iter().flat_map(|(_, v)| v.clone()).collect::<Vec<_>>()}),
            Some(k) => serde_json::json!({
                "def": want,
                "groupBy": k,
                "count": total,
                "groups": ordered.iter().map(|(v, e)| serde_json::json!({"value": v, "count": e.len(), "elements": e})).collect::<Vec<_>>(),
            }),
        };
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
        return 0;
    }
    match group_by {
        None => {
            println!("# {want}\n\n{total} application(s)\n");
            for (_, v) in &ordered {
                for q in v {
                    println!("- {q}");
                }
            }
        }
        Some(k) => {
            println!("# {want} by {k}\n\n{total} application(s)\n");
            for (v, e) in &ordered {
                println!("## {} ({})\n", v.as_deref().unwrap_or("(unset)"), e.len());
                for q in e {
                    println!("- {q}");
                }
                println!();
            }
        }
    }
    0
}
