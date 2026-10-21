//! Typed tagged values of a stereotype (`REQ-TRS-META-010..014`, GH #375).
//!
//! A `MetadataDef` declares its tagged values as `features:`. Beyond the name (`W045`), a feature
//! may carry `typedBy`, `unit`, `range`, `enumValues`, `multiplicity` and `default`, and the
//! definition may carry `constraints:`. Everything here is opt-in: a feature that declares none of
//! them is never checked beyond its name.

use serde_yaml::Value as Yaml;

use crate::expr::{self, Quantity, Value};

/// A declared tagged value.
#[derive(Debug, Clone, Default)]
pub struct FeatureSpec {
    pub name: String,
    pub type_ref: Option<String>,
    pub unit: Option<String>,
    pub range: Option<(f64, f64)>,
    pub enum_values: Option<Vec<String>>,
    /// Lower and upper bound (`None` = unbounded). Default `0..1`.
    pub mult: (usize, Option<usize>),
    /// Whether `multiplicity:` was written; an undeclared one is never enforced.
    pub mult_declared: bool,
    pub default: Option<Yaml>,
}

/// The kind of value a feature's `typedBy` asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Integer,
    Real,
    Boolean,
    Text,
    /// A reference to an element of this element-type name (`PartDef`, `Requirement`, …).
    Element(String),
    /// Declared, but not something this check understands.
    Unchecked,
}

impl FeatureSpec {
    pub fn kind(&self) -> Kind {
        let Some(t) = self.type_ref.as_deref() else { return Kind::Unchecked };
        match t.strip_prefix("ScalarValues::").unwrap_or(t) {
            "Integer" | "Natural" | "Int" => Kind::Integer,
            "Real" | "Number" | "Rational" => Kind::Real,
            "Boolean" => Kind::Boolean,
            "String" => Kind::Text,
            other if other.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && !other.contains("::") && crate::element::ElementType::ALL.iter().any(|t| format!("{t:?}") == other) => {
                Kind::Element(other.to_string())
            }
            _ => Kind::Unchecked,
        }
    }

    /// Whether the feature asks for any value-level check at all.
    pub fn is_typed(&self) -> bool {
        self.kind() != Kind::Unchecked || self.unit.is_some() || self.range.is_some() || self.enum_values.is_some()
    }
}

fn key(s: &str) -> Yaml {
    Yaml::String(s.to_string())
}

fn parse_mult(s: &str) -> Option<(usize, Option<usize>)> {
    let s = s.trim().trim_matches(|c| c == '[' || c == ']');
    if s == "*" {
        return Some((0, None));
    }
    match s.split_once("..") {
        Some((lo, "*")) => Some((lo.trim().parse().ok()?, None)),
        Some((lo, hi)) => Some((lo.trim().parse().ok()?, Some(hi.trim().parse().ok()?))),
        None => {
            let n = s.parse().ok()?;
            Some((n, Some(n)))
        }
    }
}

fn parse_range(s: &str) -> Option<(f64, f64)> {
    let (lo, hi) = s.split_once("..")?;
    Some((lo.trim().parse().ok()?, hi.trim().parse().ok()?))
}

fn yaml_text(v: &Yaml) -> Option<String> {
    match v {
        Yaml::String(s) => Some(s.clone()),
        Yaml::Number(n) => Some(n.to_string()),
        Yaml::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// The declared features of a `MetadataDef`, in file order.
pub fn feature_specs(features: &Option<Vec<Yaml>>) -> Vec<FeatureSpec> {
    features
        .iter()
        .flatten()
        .filter_map(|f| {
            let Yaml::Mapping(m) = f else { return None };
            let name = m.get(key("name"))?.as_str()?.to_string();
            let s = |k: &str| m.get(key(k)).and_then(Yaml::as_str).map(str::to_string);
            Some(FeatureSpec {
                name,
                type_ref: s("typedBy").or_else(|| s("type")),
                unit: s("unit"),
                range: m.get(key("range")).and_then(|r| match r {
                    Yaml::String(t) => parse_range(t),
                    _ => None,
                }),
                enum_values: m.get(key("enumValues")).and_then(Yaml::as_sequence).map(|q| q.iter().filter_map(yaml_text).collect()),
                mult: m
                    .get(key("multiplicity"))
                    .and_then(|v| yaml_text(v))
                    .and_then(|t| parse_mult(&t))
                    .unwrap_or((0, Some(1))),
                mult_declared: m.get(key("multiplicity")).and_then(|v| yaml_text(v)).and_then(|t| parse_mult(&t)).is_some(),
                default: m.get(key("default")).cloned(),
            })
        })
        .collect()
}

/// The values a key carries, flattening a YAML list.
pub fn values_of(v: &Yaml) -> Vec<&Yaml> {
    match v {
        Yaml::Sequence(q) => q.iter().collect(),
        other => vec![other],
    }
}

/// Convert a scalar tagged value to the expression language's value (a string with a unit
/// becomes a dimensioned number).
pub fn to_expr_value(v: &Yaml) -> Option<Value> {
    match v {
        Yaml::Bool(b) => Some(Value::Bool(*b)),
        Yaml::Number(n) => Some(Value::Num(Quantity::scalar(n.as_f64()?))),
        Yaml::String(s) => Some(Value::Str(s.clone())),
        _ => None,
    }
}

/// One finding about a value: `(code, message)`.
pub type Finding = (&'static str, String);

/// Check one scalar value against a feature. Reference kinds are checked by the caller (they need
/// the resolver); here they only have to be text.
pub fn check_value(def: &str, spec: &FeatureSpec, v: &Yaml) -> Vec<Finding> {
    let mut out = Vec::new();
    let bad = |what: String| ("E322", format!("tagged value '{}' of stereotype '{def}' {what}", spec.name));
    let kind = spec.kind();
    let mut number: Option<f64> = None;
    match (&kind, v) {
        (Kind::Integer, Yaml::Number(n)) if n.is_i64() || n.is_u64() => number = n.as_f64(),
        (Kind::Integer, Yaml::Number(n)) if n.as_f64().is_some_and(|f| f.fract() == 0.0) => number = n.as_f64(),
        (Kind::Integer, other) => out.push(bad(format!("must be an Integer, found {}", describe(other)))),
        (Kind::Real, Yaml::Number(n)) => number = n.as_f64(),
        (Kind::Real, Yaml::String(s)) if spec.unit.is_some() => match parse_quantity(s, spec.unit.as_deref().unwrap()) {
            Ok(x) => number = Some(x),
            Err(e) => out.push(bad(e)),
        },
        (Kind::Real, other) => out.push(bad(format!("must be a Real, found {}", describe(other)))),
        (Kind::Boolean, Yaml::Bool(_)) => {}
        (Kind::Boolean, other) => out.push(bad(format!("must be a Boolean, found {}", describe(other)))),
        (Kind::Text, Yaml::String(_) | Yaml::Number(_) | Yaml::Bool(_)) => {}
        (Kind::Text, other) => out.push(bad(format!("must be a String, found {}", describe(other)))),
        (Kind::Element(_), Yaml::String(_)) => {}
        (Kind::Element(t), other) => out.push(bad(format!("must be a reference to a {t}, found {}", describe(other)))),
        (Kind::Unchecked, Yaml::Number(n)) => number = n.as_f64(),
        (Kind::Unchecked, _) => {}
    }
    // A unit on a numeric type: a quantity given as text was handled above; a bare number is
    // taken to be in the declared unit. A text quantity under an Integer type is a type error.
    if let (Kind::Unchecked | Kind::Integer, Yaml::String(s), Some(u)) = (&kind, v, spec.unit.as_deref()) {
        if kind == Kind::Unchecked {
            match parse_quantity(s, u) {
                Ok(x) => number = Some(x),
                Err(e) => out.push(bad(e)),
            }
        }
    }
    if let (Some(n), Some((lo, hi))) = (number, spec.range) {
        if n < lo || n > hi {
            out.push(("E323", format!("tagged value '{}' of stereotype '{def}' is {} which is outside the range {}..{}", spec.name, expr::format_number(n), expr::format_number(lo), expr::format_number(hi))));
        }
    }
    if let (Some(allowed), Some(t)) = (&spec.enum_values, yaml_text(v)) {
        if !allowed.contains(&t) {
            out.push(("E324", format!("tagged value '{}' of stereotype '{def}' is '{t}', not one of [{}]", spec.name, allowed.join(", "))));
        }
    }
    out
}

fn describe(v: &Yaml) -> String {
    match v {
        Yaml::String(s) => format!("'{s}'"),
        Yaml::Number(n) => n.to_string(),
        Yaml::Bool(b) => b.to_string(),
        Yaml::Sequence(_) => "a list".into(),
        Yaml::Mapping(_) => "a mapping".into(),
        _ => "nothing".into(),
    }
}

/// `"3 s"` read against the feature's `unit`: the dimension must agree; the number is returned in
/// that unit.
fn parse_quantity(text: &str, unit: &str) -> Result<f64, String> {
    match expr::parse_value(text) {
        Some(Value::Num(q)) => expr::to_unit(&q, unit.strip_prefix("SI::").unwrap_or(unit)).map_err(|e| e.to_string()),
        _ => Err(format!("must be a quantity in {unit}, found '{text}'")),
    }
}

/// Count and multiplicity check of one key's values (`E325`), with `present` the number of values
/// the application gives (a default counts as one).
pub fn check_multiplicity(def: &str, spec: &FeatureSpec, present: usize) -> Option<Finding> {
    let (lo, hi) = spec.mult;
    let over = hi.is_some_and(|h| present > h);
    if present >= lo && !over {
        return None;
    }
    let want = match hi {
        Some(h) if h == lo => format!("{lo}"),
        Some(h) => format!("{lo}..{h}"),
        None => format!("{lo}..*"),
    };
    Some(("E325", format!("tagged value '{}' of stereotype '{def}' has {present} value(s) but its multiplicity is {want}", spec.name)))
}

/// Rewrite `A implies B` (top level, once) as `not (A) or (B)`.
fn rewrite_implies(src: &str) -> String {
    let mut depth = 0i32;
    let b = src.as_bytes();
    let mut i = 0;
    let mut in_str: Option<u8> = None;
    while i < b.len() {
        let c = b[i];
        match in_str {
            Some(q) => {
                if c == q {
                    in_str = None;
                }
            }
            None => match c {
                b'"' | b'\'' => in_str = Some(c),
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ if depth == 0 && src[i..].starts_with(" implies ") => {
                    let (a, rest) = (&src[..i], &src[i + " implies ".len()..]);
                    return format!("not ({}) or ({})", a.trim(), rewrite_implies(rest).trim());
                }
                _ => {}
            },
        }
        i += 1;
    }
    src.to_string()
}

fn meta_call_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\bmeta\(\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap())
}

fn has_call_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\bhas\(\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\)").unwrap())
}

/// The stereotype names a constraint reads through `has(Def)` / `meta(Def).key`.
pub fn referenced_defs(src: &str) -> Vec<String> {
    let mut out: Vec<String> = meta_call_re().captures_iter(src).map(|c| c[1].to_string()).collect();
    out.extend(has_call_re().captures_iter(src).map(|c| c[1].to_string()));
    out.dedup();
    out
}

/// `has(Def)` / `meta(Def).key` become the names `has.Def` / `meta.Def.key`, which the evaluator
/// resolves against the element in scope (`REQ-TRS-META-023`).
fn rewrite_calls(src: &str) -> Result<String, String> {
    let a = meta_call_re().replace_all(src, "meta.$1.$2");
    let b = has_call_re().replace_all(&a, "has.$1").into_owned();
    let stray = regex::Regex::new(r"\b(meta|has)\s*\(").unwrap();
    if stray.is_match(&b) {
        return Err("malformed call: expected `has(Def)` or `meta(Def).key`".to_string());
    }
    Ok(b)
}

/// Parse a `constraints:` entry; an error is the message to report on the definition.
pub fn parse_constraint(src: &str) -> Result<expr::Expr, String> {
    let rewritten = rewrite_calls(&rewrite_implies(src))?;
    expr::parse(&rewritten).map_err(|e| e.to_string())
}

/// The element in scope for `has()` / `meta()`.
pub struct Scope<'a> {
    pub elements: &'a [crate::element::RawElement],
    pub resolver: &'a crate::resolver::Resolver,
    pub carrier: &'a crate::element::RawElement,
}

/// Outcome of evaluating a constraint over one application.
#[derive(Debug, PartialEq)]
pub enum Verdict {
    Holds,
    Violated,
    /// A value it needs is absent (and has no neutral stand-in) or the types do not meet.
    Undecided,
}

/// Evaluate `e` over an application's values. An absent String is the empty string and an absent
/// Boolean is false, so `goal != ""` is decidable; an absent number leaves the constraint undecided.
/// With a `scope`, `has(Def)` and `meta(Def).key` read the carrying element: an absent stereotype
/// or value is *absent*, and any comparison with it is false (never an error).
pub fn evaluate(e: &expr::Expr, specs: &[FeatureSpec], values: &[(String, Yaml)], scope: Option<&Scope>) -> Verdict {
    let apps = scope.map(|sc| effective_applications(sc.elements, sc.resolver, sc.carrier)).unwrap_or_default();
    let applies = |def: &str| -> Option<&EffectiveApp> {
        let sc = scope?;
        let want = find_metadata_def(sc.elements, sc.resolver, def)?.qualified_name.clone();
        apps.iter().find(|a| a.def == want)
    };
    let lookup = |n: &str| -> Option<Value> {
        if let Some(def) = n.strip_prefix("has.") {
            return Some(Value::Bool(applies(def).is_some()));
        }
        if let Some(rest) = n.strip_prefix("meta.") {
            let (def, key) = rest.rsplit_once('.')?;
            let app = applies(def)?;
            return to_expr_value(&app.values.iter().find(|(k, _)| k == key)?.1);
        }
        if let Some((_, v)) = values.iter().find(|(k, _)| k == n) {
            return to_expr_value(v);
        }
        let spec = specs.iter().find(|s| s.name == n)?;
        if let Some(d) = &spec.default {
            return to_expr_value(d);
        }
        match spec.kind() {
            Kind::Text | Kind::Element(_) => Some(Value::Str(String::new())),
            Kind::Boolean => Some(Value::Bool(false)),
            _ => None,
        }
    };
    match eval_bool(e, &lookup) {
        Ok(true) => Verdict::Holds,
        Ok(false) => Verdict::Violated,
        Err(_) => Verdict::Undecided,
    }
}

/// Boolean evaluation in which a comparison over an absent `meta()` value is false.
fn eval_bool(e: &expr::Expr, lookup: &dyn Fn(&str) -> Option<Value>) -> Result<bool, expr::ExprError> {
    use expr::{Expr, ExprError};
    match e {
        Expr::Bin("and", l, r) => Ok(eval_bool(l, lookup)? && eval_bool(r, lookup)?),
        Expr::Bin("or", l, r) => Ok(eval_bool(l, lookup)? || eval_bool(r, lookup)?),
        Expr::Unary("not", x) => Ok(!eval_bool(x, lookup)?),
        Expr::Bin("==" | "!=" | "<" | "<=" | ">" | ">=", ..) => match e.eval(lookup) {
            Ok(Value::Bool(b)) => Ok(b),
            Err(ExprError::Unbound(n)) if n.starts_with("meta.") => Ok(false),
            Ok(other) => Err(ExprError::Type(format!("expected a boolean, found {}", other.kind()))),
            Err(err) => Err(err),
        },
        other => match other.eval(lookup)? {
            Value::Bool(b) => Ok(b),
            v => Err(ExprError::Type(format!("expected a boolean, found {}", v.kind()))),
        },
    }
}

/// An application with its declared defaults filled in (`REQ-TRS-META-012`).
#[derive(Debug, Clone)]
pub struct EffectiveApp {
    /// The definition's qualified name when it resolves, else the name as written.
    pub def: String,
    /// Written values first (file order), then defaults for the keys left out.
    pub values: Vec<(String, Yaml)>,
    /// The keys whose value came from a default.
    pub defaulted: Vec<String>,
}

fn metadata_def<'a>(
    elements: &'a [crate::element::RawElement],
    resolver: &crate::resolver::Resolver,
    name: &str,
) -> Option<&'a crate::element::RawElement> {
    resolver
        .resolve_ref(elements, name)
        .filter(|d| matches!(d.frontmatter.element_type, Some(crate::element::ElementType::MetadataDef)))
}

/// The element's stereotype applications with defaults applied. Never written to the file.
pub fn effective_applications(
    elements: &[crate::element::RawElement],
    resolver: &crate::resolver::Resolver,
    e: &crate::element::RawElement,
) -> Vec<EffectiveApp> {
    crate::element::metadata_applications(&e.frontmatter.metadata)
        .into_iter()
        .map(|a| {
            let def = metadata_def(elements, resolver, &a.def);
            let mut values = a.values.clone();
            let mut defaulted = Vec::new();
            if let Some(d) = def {
                for spec in feature_specs(&d.frontmatter.features) {
                    if let (Some(dv), false) = (&spec.default, values.iter().any(|(k, _)| k == &spec.name)) {
                        values.push((spec.name.clone(), dv.clone()));
                        defaulted.push(spec.name);
                    }
                }
            }
            EffectiveApp { def: def.map(|d| d.qualified_name.clone()).unwrap_or(a.def), values, defaulted }
        })
        .collect()
}

/// Reference-typed tagged values an element holds: `(Stereotype.key, target as written)`.
/// Feeds `refs`, `impact` and the delete-referrer guard (`REQ-TRS-META-013`).
pub fn reference_values(
    elements: &[crate::element::RawElement],
    resolver: &crate::resolver::Resolver,
    e: &crate::element::RawElement,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for a in crate::element::metadata_applications(&e.frontmatter.metadata) {
        let Some(def) = metadata_def(elements, resolver, &a.def) else { continue };
        let label = def.frontmatter.name.clone().unwrap_or_else(|| a.def.clone());
        for spec in feature_specs(&def.frontmatter.features) {
            if !matches!(spec.kind(), Kind::Element(_)) {
                continue;
            }
            for (k, v) in &a.values {
                if k == &spec.name {
                    for x in values_of(v) {
                        if let Some(s) = x.as_str() {
                            out.push((format!("{label}.{k}"), s.to_string()));
                        }
                    }
                }
            }
        }
    }
    out
}

/// A `MetadataDef` by reference, else by unique last `::` segment (`Safety` for `Pkg::Safety`).
pub fn find_metadata_def<'a>(
    elements: &'a [crate::element::RawElement],
    resolver: &crate::resolver::Resolver,
    name: &str,
) -> Option<&'a crate::element::RawElement> {
    if let Some(d) = metadata_def(elements, resolver, name) {
        return Some(d);
    }
    let mut hits = elements.iter().filter(|e| {
        matches!(e.frontmatter.element_type, Some(crate::element::ElementType::MetadataDef)) && e.qualified_name.rsplit("::").next() == Some(name)
    });
    let first = hits.next()?;
    hits.next().is_none().then_some(first)
}
