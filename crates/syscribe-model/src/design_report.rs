//! The detailed-design report of one software part (`REQ-TRS-DREP-001`, GH #371): interface, behaviour, data,
//! resources, code, requirements and tests, as JSON and as Markdown.

use serde_json::{json, Value};
use serde_yaml::Value as Yaml;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;
use crate::results::{testcase_verdict, ResultsData};

fn text(v: &Yaml) -> String {
    match v {
        Yaml::String(s) => s.clone(),
        Yaml::Bool(b) => b.to_string(),
        Yaml::Number(n) => n.to_string(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

fn s<'a>(m: &'a serde_yaml::Mapping, k: &str) -> Option<&'a str> {
    m.get(k).and_then(Yaml::as_str)
}

fn operations(owner: &RawElement, out: &mut Vec<Value>) {
    for op in owner.frontmatter.operations.iter().flatten().filter_map(Yaml::as_mapping) {
        let Some(name) = s(op, "name") else { continue };
        let params: Vec<Value> = op
            .get("parameters")
            .and_then(Yaml::as_sequence)
            .into_iter()
            .flatten()
            .filter_map(Yaml::as_mapping)
            .map(|p| json!({"name": s(p, "name"), "type": s(p, "typedBy").or_else(|| s(p, "type")), "direction": s(p, "direction").unwrap_or("in"), "unit": s(p, "unit")}))
            .collect();
        out.push(json!({"owner": owner.qualified_name, "name": name, "parameters": params, "returnType": s(op, "returnType"), "isAsync": op.get("isAsync").and_then(Yaml::as_bool).unwrap_or(false), "realizedBy": s(op, "realizedBy")}));
    }
}

/// Build the report for the software part `reference`. `Err` is a usage error.
pub fn build(elements: &[RawElement], resolver: &Resolver, results: Option<&ResultsData>, reference: &str) -> Result<Value, String> {
    let part = resolver.resolve_ref(elements, reference).ok_or_else(|| format!("'{reference}' does not resolve to an element"))?;
    let fm = &part.frontmatter;
    if !matches!(fm.element_type, Some(ElementType::PartDef | ElementType::Part)) || fm.domain.as_deref() != Some("software") {
        return Err(format!("'{}' is not a software PartDef or Part (`domain: software`)", part.qualified_name));
    }

    let mut interface = Vec::new();
    operations(part, &mut interface);
    let mut data = Vec::new();
    for f in fm.features.iter().flatten().filter_map(Yaml::as_mapping) {
        let Some(name) = s(f, "name") else { continue };
        let ty = s(f, "typedBy").or_else(|| s(f, "type"));
        let target = ty.and_then(|t| resolver.resolve_ref(elements, t));
        match target.map(|t| t.frontmatter.element_type.clone()) {
            Some(Some(ElementType::PortDef | ElementType::InterfaceDef | ElementType::Port | ElementType::Interface)) => operations(target.unwrap(), &mut interface),
            _ => data.push(json!({
                "name": name,
                "type": ty,
                "unit": s(f, "unit"),
                "range": target.and_then(|t| t.frontmatter.range.as_ref()).map(text),
            })),
        }
    }

    let mut behaviour: Vec<String> = interface
        .iter()
        .filter_map(|o| o["realizedBy"].as_str())
        .filter_map(|r| resolver.resolve_ref(elements, r))
        .map(|a| a.qualified_name.clone())
        .collect();
    behaviour.sort();
    behaviour.dedup();

    let mut resources = serde_json::Map::new();
    for (k, v) in [("ram", &fm.ram), ("flash", &fm.flash), ("nvm", &fm.nvm), ("stack", &fm.stack), ("period", &fm.period), ("deadline", &fm.deadline), ("priority", &fm.priority)] {
        if let Some(v) = v {
            resources.insert(k.into(), json!(text(v)));
        }
    }
    if let Some(w) = &fm.wcet {
        resources.insert("wcet".into(), json!(w));
    }

    let satisfied: Vec<&RawElement> = fm.satisfies.iter().flatten().filter_map(|r| resolver.resolve_ref(elements, r)).collect();
    let requirements: Vec<String> = satisfied.iter().map(|r| r.frontmatter.id.clone().unwrap_or_else(|| r.qualified_name.clone())).collect();
    let mut tests: Vec<Value> = elements
        .iter()
        .filter(|t| t.frontmatter.element_type == Some(ElementType::TestCase))
        .filter(|t| {
            t.frontmatter.verifies.iter().flatten().filter_map(|r| resolver.resolve_ref(elements, r)).any(|v| v.qualified_name == part.qualified_name || satisfied.iter().any(|r| r.qualified_name == v.qualified_name))
        })
        .map(|t| json!({"id": t.frontmatter.id.clone().unwrap_or_else(|| t.qualified_name.clone()), "level": t.frontmatter.test_level, "verdict": testcase_verdict(t, results).as_str()}))
        .collect();
    tests.sort_by_key(|t| t["id"].as_str().unwrap_or("").to_string());

    Ok(json!({
        "element": part.qualified_name,
        "level": fm.level,
        "description": part.doc.trim(),
        "interface": interface,
        "behaviour": behaviour,
        "data": data,
        "resources": Value::Object(resources),
        "code": fm.implemented_by.clone().unwrap_or_default(),
        "requirements": requirements,
        "tests": tests,
    }))
}

fn section(out: &mut String, title: &str, lines: Vec<String>) {
    out.push_str(&format!("\n## {title}\n\n"));
    if lines.is_empty() {
        out.push_str("None.\n");
    }
    for l in lines {
        out.push_str(&format!("- {l}\n"));
    }
}

/// The report as Markdown.
pub fn render(r: &Value) -> String {
    let mut out = format!("# {}\n\n", r["element"].as_str().unwrap_or(""));
    out.push_str(&format!("Level: {}\n", r["level"].as_str().unwrap_or("(not declared)")));
    if let Some(d) = r["description"].as_str().filter(|d| !d.is_empty()) {
        out.push_str(&format!("\n{d}\n"));
    }
    let arr = |k: &str| r[k].as_array().cloned().unwrap_or_default();
    let st = |v: &Value| v.as_str().unwrap_or("").to_string();
    section(&mut out, "Interface", arr("interface").iter().map(|o| {
        let ps = o["parameters"].as_array().map(|p| p.iter().map(|p| format!("{} {}: {}", st(&p["direction"]), st(&p["name"]), p["type"].as_str().unwrap_or("?"))).collect::<Vec<_>>().join(", ")).unwrap_or_default();
        let ret = o["returnType"].as_str().map(|t| format!(" -> {t}")).unwrap_or_default();
        format!("`{}({ps}){ret}` ({})", st(&o["name"]), st(&o["owner"]))
    }).collect());
    section(&mut out, "Behaviour", arr("behaviour").iter().map(st).collect());
    section(&mut out, "Data", arr("data").iter().map(|d| {
        let unit = d["unit"].as_str().map(|u| format!(" [{u}]")).unwrap_or_default();
        let range = d["range"].as_str().map(|u| format!(", range {u}")).unwrap_or_default();
        format!("{}: {}{unit}{range}", st(&d["name"]), d["type"].as_str().unwrap_or("?"))
    }).collect());
    section(&mut out, "Resources", r["resources"].as_object().map(|m| m.iter().map(|(k, v)| format!("{k}: {}", st(v))).collect()).unwrap_or_default());
    section(&mut out, "Code", arr("code").iter().map(|c| format!("`{}`", st(c))).collect());
    section(&mut out, "Requirements", arr("requirements").iter().map(st).collect());
    section(&mut out, "Tests", arr("tests").iter().map(|t| format!("{} ({}): {}", st(&t["id"]), t["level"].as_str().unwrap_or("?"), st(&t["verdict"]))).collect());
    out
}
