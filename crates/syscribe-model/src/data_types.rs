//! Data types with value constraints (`REQ-TRS-DATA-001`..`-003`, GH #366): `range:`, `pattern:` and `default:` on
//! an `AttributeDef`, a subtype that stays within them, and values bound to features typed by it.
//!
//! Opt-in: a model that declares none of the three fields sees nothing new.

use serde_yaml::Value as Yaml;

use crate::element::{ElementType, RawElement};
use crate::resolver::Resolver;

/// `(is_error, code, file, message)`.
pub type Finding = (bool, &'static str, String, String);

/// A numeric interval with open or closed ends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
    pub lo_open: bool,
    pub hi_open: bool,
}

impl Interval {
    /// `lo..hi` (closed) or `[lo, hi]`, `(lo, hi)`, `[lo, hi)`, `(lo, hi]`; `None` if it does not parse or `lo > hi`.
    pub fn parse(text: &str) -> Option<Interval> {
        let t = text.trim();
        let num = |s: &str| s.trim().parse::<f64>().ok().filter(|v| v.is_finite());
        let (lo, hi, lo_open, hi_open) = if let Some(first) = t.chars().next().filter(|c| matches!(c, '[' | '(')) {
            let last = t.chars().last()?;
            if !matches!(last, ']' | ')') || t.len() < 2 {
                return None;
            }
            let inner = &t[1..t.len() - 1];
            let (a, b) = inner.split_once(',')?;
            (num(a)?, num(b)?, first == '(', last == ')')
        } else {
            let (a, b) = t.split_once("..")?;
            (num(a)?, num(b)?, false, false)
        };
        (lo <= hi).then_some(Interval { lo, hi, lo_open, hi_open })
    }

    pub fn contains(&self, x: f64) -> bool {
        (x > self.lo || (x == self.lo && !self.lo_open)) && (x < self.hi || (x == self.hi && !self.hi_open))
    }

    /// Whether every value of `self` is in `outer`.
    pub fn within(&self, outer: &Interval) -> bool {
        let lo_ok = self.lo > outer.lo || (self.lo == outer.lo && (!outer.lo_open || self.lo_open));
        let hi_ok = self.hi < outer.hi || (self.hi == outer.hi && (!outer.hi_open || self.hi_open));
        lo_ok && hi_ok
    }

    fn text(&self) -> String {
        let n = |v: f64| crate::expr::format_number(v);
        format!("{}{}, {}{}", if self.lo_open { '(' } else { '[' }, n(self.lo), n(self.hi), if self.hi_open { ')' } else { ']' })
    }
}

fn pattern_matches(pattern: &str, value: &str) -> bool {
    regex::Regex::new(&format!("^(?:{pattern})$")).is_ok_and(|re| re.is_match(value))
}

/// The nearest `range:`, `pattern:` and `unit:` up an `AttributeDef`'s supertype chain (cycle-safe).
#[derive(Default)]
struct Effective {
    range: Option<(Interval, String, String)>,
    pattern: Option<(String, String)>,
    unit: Option<String>,
}

fn supertypes(def: &RawElement) -> Vec<String> {
    match def.frontmatter.supertype.as_ref() {
        Some(Yaml::String(s)) => vec![s.clone()],
        Some(Yaml::Sequence(q)) => q.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

fn effective(def: &RawElement, elements: &[RawElement], resolver: &Resolver) -> Effective {
    let mut out = Effective::default();
    let mut seen: Vec<String> = Vec::new();
    let mut cur = Some(def);
    while let Some(d) = cur {
        if seen.contains(&d.qualified_name) {
            break;
        }
        seen.push(d.qualified_name.clone());
        if out.range.is_none() {
            if let Some(i) = d.frontmatter.range.as_ref().and_then(Yaml::as_str).and_then(Interval::parse) {
                out.range = Some((i, d.qualified_name.clone(), d.frontmatter.range.as_ref().and_then(Yaml::as_str).unwrap_or_default().to_string()));
            }
        }
        if out.pattern.is_none() {
            if let Some(p) = d.frontmatter.pattern.as_ref().and_then(Yaml::as_str).filter(|p| regex::Regex::new(p).is_ok()) {
                out.pattern = Some((p.to_string(), d.qualified_name.clone()));
            }
        }
        if out.unit.is_none() {
            out.unit = d.frontmatter.unit.clone();
        }
        cur = supertypes(d).iter().find_map(|s| resolver.resolve_ref(elements, s)).filter(|s| s.frontmatter.element_type == Some(ElementType::AttributeDef));
    }
    out
}

/// A value as a number in the constraint's unit, or why it is not one.
fn number_of(v: &Yaml, unit: Option<&str>) -> Result<f64, String> {
    let text = match v {
        Yaml::Number(n) => return n.as_f64().ok_or_else(|| "is not a number".to_string()),
        Yaml::String(s) => s.trim().to_string(),
        _ => return Err("is not a number".into()),
    };
    if let Ok(x) = text.parse::<f64>() {
        return Ok(x);
    }
    match crate::expr::parse_value(&text) {
        Some(crate::expr::Value::Num(q)) => match unit {
            Some(u) => crate::expr::to_unit(&q, u.trim().strip_prefix("SI::").unwrap_or(u.trim())).map_err(|e| format!("cannot be read in {u}: {e}")),
            None => Ok(q.v),
        },
        _ => Err("is not a number".into()),
    }
}

fn text_of(v: &Yaml) -> String {
    match v {
        Yaml::String(s) => s.clone(),
        Yaml::Number(n) => n.to_string(),
        Yaml::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Why `v` violates `eff`, if it does.
fn violation(v: &Yaml, eff: &Effective) -> Option<String> {
    if let Some((range, owner, raw)) = &eff.range {
        match number_of(v, eff.unit.as_deref()) {
            Ok(x) if range.contains(x) => {}
            Ok(_) => return Some(format!("is outside the range {raw} of {owner}")),
            Err(why) => return Some(format!("{why}, but {owner} has a numeric range")),
        }
    }
    if let Some((pat, owner)) = &eff.pattern {
        let t = text_of(v);
        if !pattern_matches(pat, &t) {
            return Some(format!("does not match the pattern '{pat}' of {owner}"));
        }
    }
    None
}

pub fn check(elements: &[RawElement], resolver: &Resolver) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    for def in elements.iter().filter(|e| e.frontmatter.element_type == Some(ElementType::AttributeDef)) {
        let fm = &def.frontmatter;
        let file = def.file_path.clone();
        let name = &def.qualified_name;

        let own_range = fm.range.as_ref().map(|r| (r.as_str(), r.as_str().and_then(Interval::parse)));
        match own_range {
            Some((Some(raw), None)) => out.push((true, "E680", file.clone(), format!("{name}: range '{raw}' is not an interval (use `lo..hi`, `[lo, hi]`, `(lo, hi)`, `[lo, hi)` or `(lo, hi]` with lo <= hi)"))),
            Some((None, _)) => out.push((true, "E680", file.clone(), format!("{name}: range must be text such as `0..250`"))),
            _ => {}
        }
        let own_pattern = fm.pattern.as_ref().and_then(Yaml::as_str);
        if fm.pattern.is_some() && own_pattern.is_none() {
            out.push((true, "E680", file.clone(), format!("{name}: pattern must be text holding a regular expression")));
        }
        if let Some(p) = own_pattern {
            if let Err(e) = regex::Regex::new(p) {
                out.push((true, "E680", file.clone(), format!("{name}: pattern '{p}' is not a valid regular expression: {e}")));
            }
        }

        // narrowing against the nearest supertype that constrains
        let parent = supertypes(def).iter().find_map(|s| resolver.resolve_ref(elements, s)).filter(|s| s.frontmatter.element_type == Some(ElementType::AttributeDef));
        if let Some(parent) = parent {
            let peff = effective(parent, elements, resolver);
            if let (Some((_, Some(own))), Some((pr, owner, _))) = (own_range, &peff.range) {
                if !own.within(pr) {
                    out.push((true, "E683", file.clone(), format!("{name}: range {} is not within {} of its supertype {owner}", own.text(), pr.text())));
                }
            }
            if let (Some(own), Some((pp, owner))) = (own_pattern, &peff.pattern) {
                if own != pp {
                    out.push((true, "E683", file.clone(), format!("{name}: pattern '{own}' differs from '{pp}' of its supertype {owner}, and a pattern cannot be shown to narrow another")));
                }
            }
        }

        // the default must satisfy what the type is
        if let Some(d) = &fm.default {
            let eff = effective(def, elements, resolver);
            if let Some(why) = violation(d, &eff) {
                out.push((true, "E681", file.clone(), format!("{name}: default {} {why}", text_of(d))));
            }
        }
    }

    // values bound to features typed by a constrained AttributeDef
    for e in elements.iter() {
        let file = e.file_path.clone();
        let mut check_bound = |fname: &str, typed_by: Option<&str>, value: Option<&Yaml>| {
            let (Some(t), Some(v)) = (typed_by, value) else { return };
            let Some(def) = resolver.resolve_ref(elements, t).filter(|d| d.frontmatter.element_type == Some(ElementType::AttributeDef)) else { return };
            let eff = effective(def, elements, resolver);
            if let Some(why) = violation(v, &eff) {
                out.push((true, "E682", file.clone(), format!("feature '{fname}' of '{}' is bound to {}, which {why}", e.qualified_name, text_of(v))));
            }
        };
        for f in e.frontmatter.features.iter().flatten().filter_map(|f| f.as_mapping()) {
            let name = f.get("name").and_then(Yaml::as_str).unwrap_or("?");
            let ty = f.get("typedBy").or_else(|| f.get("type")).and_then(Yaml::as_str);
            check_bound(name, ty, f.get("value"));
        }
        if e.frontmatter.element_type == Some(ElementType::Attribute) {
            let ty = e.frontmatter.typed_by.as_ref().and_then(|t| match t {
                Yaml::String(s) => Some(s.as_str()),
                Yaml::Sequence(q) => q.first().and_then(Yaml::as_str),
                _ => None,
            });
            let name = e.frontmatter.name.clone().unwrap_or_else(|| e.qualified_name.clone());
            check_bound(&name, ty, e.frontmatter.value.as_ref());
        }
    }
    out
}
